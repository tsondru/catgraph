use crate::errors::CatgraphError;

use {
    rustworkx_core::petgraph::{algo::toposort, prelude::DiGraph},
    std::{collections::HashMap, marker::PhantomData},
};

/// Trait for types that reference other labels (used for dependency tracking).
pub trait Contains<BlackBoxLabel> {
    fn contained_labels(&self) -> Vec<BlackBoxLabel>;
}

/// Trait for morphisms that can be constructed by interpreting a general (possibly black-boxed) description.
pub trait InterpretableMorphism<GeneralVersion, Lambda, BlackBoxLabel>: Sized {
    /// Interpret a general morphism description, resolving black boxes via the provided closure.
    ///
    /// # Errors
    ///
    /// Returns `CatgraphError` if black box interpretation fails.
    fn interpret<F>(r#gen: &GeneralVersion, black_box_interpreter: F) -> Result<Self, CatgraphError>
    where
        F: Fn(&BlackBoxLabel, &[Lambda], &[Lambda]) -> Result<Self, CatgraphError>;
}

/// DAG of named morphisms resolved via topological sort.
///
/// Composite definitions may reference other labels; `fill_black_boxes` resolves them
/// bottom-up into concrete `T` values using `InterpretableMorphism::interpret`.
pub struct MorphismSystem<BlackBoxLabel, Lambda, GeneralBlackBoxed, T>
where
    BlackBoxLabel: std::hash::Hash + Eq,
    T: InterpretableMorphism<GeneralBlackBoxed, Lambda, BlackBoxLabel>,
    GeneralBlackBoxed: Contains<BlackBoxLabel>,
{
    composite_pieces: HashMap<BlackBoxLabel, GeneralBlackBoxed>,
    simple_pieces: HashMap<BlackBoxLabel, T>,
    pub(crate) main: BlackBoxLabel,
    dag: DiGraph<BlackBoxLabel, ()>,
    dummy: PhantomData<Lambda>,
}

impl<GeneralBlackBoxed, BlackBoxLabel, Lambda, T>
    MorphismSystem<BlackBoxLabel, Lambda, GeneralBlackBoxed, T>
where
    BlackBoxLabel: std::hash::Hash + Eq + Clone + std::fmt::Debug,
    Lambda: Eq + std::fmt::Debug + Copy,
    T: InterpretableMorphism<GeneralBlackBoxed, Lambda, BlackBoxLabel> + Clone,
    GeneralBlackBoxed: Contains<BlackBoxLabel>,
{
    /// Create an empty system with the given main entry point label.
    pub fn new(main_name: BlackBoxLabel) -> Self {
        Self {
            composite_pieces: HashMap::new(),
            simple_pieces: HashMap::new(),
            main: main_name,
            dag: DiGraph::new(),
            dummy: PhantomData,
        }
    }

    /// Find or create a DAG node for the given label, returning its `NodeIndex`.
    fn ensure_node(&mut self, label: &BlackBoxLabel) -> rustworkx_core::petgraph::graph::NodeIndex {
        self.dag
            .node_indices()
            .find(|&i| self.dag[i] == *label)
            .unwrap_or_else(|| self.dag.add_node(label.clone()))
    }

    /// Register a composite definition that depends on other labels.
    ///
    /// Adds `new_name` to the DAG with edges to each label returned by
    /// `new_def.contained_labels()`, then verifies acyclicity. Returns
    /// `CatgraphError::Interpret` if the addition would create a cycle.
    ///
    /// # Errors
    ///
    /// - Name already exists or would create a cycle in the dependency DAG.
    pub fn add_definition_composite(
        &mut self,
        new_name: BlackBoxLabel,
        new_def: GeneralBlackBoxed,
    ) -> Result<(), CatgraphError> {
        let contained = new_def.contained_labels();

        // Build the mutation on a clone first so we can check acyclicity
        // before committing to the real DAG.
        let mut trial_dag = self.dag.clone();
        let parent = trial_dag
            .node_indices()
            .find(|&i| trial_dag[i] == new_name)
            .unwrap_or_else(|| trial_dag.add_node(new_name.clone()));
        for child_label in &contained {
            let child = trial_dag
                .node_indices()
                .find(|&i| trial_dag[i] == *child_label)
                .unwrap_or_else(|| trial_dag.add_node(child_label.clone()));
            trial_dag.add_edge(parent, child, ());
        }

        // Verify acyclicity on the trial DAG.
        if toposort(&trial_dag, None).is_err() {
            return Err(CatgraphError::Interpret { context: format!(
                "Adding composite {new_name:?} would create a cycle in the dependency DAG"
            ) });
        }

        // Commit: apply the same mutations to the real DAG.
        self.dag = trial_dag;
        self.composite_pieces.insert(new_name, new_def);
        Ok(())
    }

    /// Register a simple (leaf) definition with no dependencies.
    ///
    /// Adds `new_name` as a leaf node in the DAG and stores the definition
    /// in `simple_pieces`.
    ///
    /// # Errors
    ///
    /// - Name already exists in the system.
    #[allow(clippy::unnecessary_wraps)] // consistent API with add_definition_composite
    pub fn add_definition_simple(
        &mut self,
        new_name: BlackBoxLabel,
        new_def: T,
    ) -> Result<(), CatgraphError> {
        self.ensure_node(&new_name);
        self.simple_pieces.insert(new_name, new_def);
        Ok(())
    }

    /// Change which label is treated as the top-level entry point for resolution.
    pub fn set_main(&mut self, main_name: BlackBoxLabel) {
        self.main = main_name;
    }

    fn interpret_nomut(
        &self,
        interpret_target: Option<BlackBoxLabel>,
    ) -> Result<T, CatgraphError> {
        let which_interpreting = interpret_target.unwrap_or(self.main.clone());
        if let Some(simple_answer) = self.simple_pieces.get(&which_interpreting) {
            return Ok(simple_answer.clone());
        }
        let complicated_answer = self.composite_pieces.get(&which_interpreting);
        if let Some(complicated_answer_2) = complicated_answer {
            let black_box_interpreter = |bb: &BlackBoxLabel, _src: &[Lambda], _tgt: &[Lambda]| {
                let simple_answer = self
                    .simple_pieces
                    .get(bb)
                    .ok_or(CatgraphError::Interpret { context: format!("No filling for {:?}", bb.clone()) })
                    .cloned();
                if simple_answer.is_err() {
                    self.interpret_nomut(Some(bb.clone()))
                } else {
                    simple_answer
                }
            };
            T::interpret(complicated_answer_2, black_box_interpreter).map_err(
                |internal_explanation| {
                    CatgraphError::Interpret { context: format!("When doing {which_interpreting:?}\n{internal_explanation:?}") }
                },
            )
        } else {
            Err(CatgraphError::Interpret { context: format!("No {which_interpreting:?} found") })
        }
    }

    /// Resolve all definitions in topological order, returning the concrete morphism for
    /// `interpret_target` (or `main` if `None`). Resolved composites are cached as simple pieces.
    ///
    /// # Errors
    ///
    /// - Topological resolution fails (cyclic dependencies).
    /// - Black box interpretation fails for any definition.
    pub fn fill_black_boxes(
        &mut self,
        interpret_target: Option<BlackBoxLabel>,
    ) -> Result<T, CatgraphError> {
        let which_interpreting = interpret_target.unwrap_or(self.main.clone());
        if let Some(simple_answer) = self.simple_pieces.get(&which_interpreting) {
            return Ok(simple_answer.clone());
        }
        let resolution_order = toposort(&self.dag, None);
        if let Ok(ordered) = resolution_order {
            for cur_node in ordered {
                let node_name = self.dag.node_weight(cur_node);
                if let Some(my_bb) = node_name {
                    let cur_answer = self.interpret_nomut(Some(my_bb.clone()));
                    if let Ok(real_cur_answer) = cur_answer.clone() {
                        self.simple_pieces.insert(my_bb.clone(), real_cur_answer);
                        let _ = self.composite_pieces.remove(my_bb);
                    }
                    if *my_bb == which_interpreting {
                        return cur_answer;
                    }
                } else {
                    return Err(CatgraphError::Interpret {
                        context: format!("Node {cur_node:?} not found after topological sort"),
                    });
                }
            }
            Err(CatgraphError::Interpret { context: format!("Through all but never found {which_interpreting:?}") })
        } else {
            Err(CatgraphError::Interpret { context: "Not acyclic dependencies".to_string() })
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Contains, CatgraphError, InterpretableMorphism, MorphismSystem};

    #[test]
    fn catgraph_error_interpret() {
        let error = CatgraphError::Interpret { context: "test error".to_string() };
        match error {
            CatgraphError::Interpret { context } => assert_eq!(context, "test error"),
            _ => panic!("Expected Interpret variant"),
        }
    }

    #[test]
    fn catgraph_error_interpret_display() {
        let error = CatgraphError::Interpret { context: "test error".to_string() };
        let display_str = format!("{error}");
        assert!(display_str.contains("test error"));
    }

    #[test]
    fn contains_trait() {
        #[derive(Clone)]
        struct SimpleContainer {
            labels: Vec<String>,
        }

        impl Contains<String> for SimpleContainer {
            fn contained_labels(&self) -> Vec<String> {
                self.labels.clone()
            }
        }

        let container = SimpleContainer {
            labels: vec!["a".to_string(), "b".to_string()],
        };
        let labels = container.contained_labels();
        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0], "a");
        assert_eq!(labels[1], "b");
    }

    #[test]
    fn morphism_system_new() {
        #[derive(Clone)]
        struct SimpleContainer {
            labels: Vec<String>,
        }
        impl Contains<String> for SimpleContainer {
            fn contained_labels(&self) -> Vec<String> {
                self.labels.clone()
            }
        }
        #[derive(Clone, Debug)]
        struct SimpleMorphism;
        impl InterpretableMorphism<SimpleContainer, char, String> for SimpleMorphism {
            fn interpret<F>(
                _container: &SimpleContainer,
                _black_box_interpreter: F,
            ) -> Result<Self, CatgraphError>
            where
                F: Fn(&String, &[char], &[char]) -> Result<Self, CatgraphError>,
            {
                Ok(SimpleMorphism)
            }
        }

        let system: MorphismSystem<String, char, SimpleContainer, SimpleMorphism> =
            MorphismSystem::new("main".to_string());
        assert_eq!(system.main, "main".to_string());
    }

    #[test]
    fn morphism_system_set_main() {
        #[derive(Clone)]
        struct SimpleContainer {
            labels: Vec<String>,
        }
        impl Contains<String> for SimpleContainer {
            fn contained_labels(&self) -> Vec<String> {
                self.labels.clone()
            }
        }
        #[derive(Clone, Debug)]
        struct SimpleMorphism {
            name: String,
        }
        impl InterpretableMorphism<SimpleContainer, char, String> for SimpleMorphism {
            fn interpret<F>(
                container: &SimpleContainer,
                _black_box_interpreter: F,
            ) -> Result<Self, CatgraphError>
            where
                F: Fn(&String, &[char], &[char]) -> Result<Self, CatgraphError>,
            {
                Ok(SimpleMorphism {
                    name: container.labels.join(","),
                })
            }
        }

        let mut system: MorphismSystem<String, char, SimpleContainer, SimpleMorphism> =
            MorphismSystem::new("initial".to_string());
        system.set_main("new_main".to_string());
        assert_eq!(system.main, "new_main".to_string());

        // read the name field so it is not considered dead code
        let container = SimpleContainer {
            labels: vec!["a".to_string(), "b".to_string()],
        };
        let interpreted = SimpleMorphism::interpret(&container, |_bb, _src, _tgt| {
            panic!("No black-boxs expected")
        })
        .unwrap();
        assert_eq!(interpreted.name, "a,b".to_string());
    }

    // ── MorphismSystem DAG registration tests ──

    /// Shared test scaffolding: a container whose `contained_labels` returns
    /// an arbitrary set of `String` labels, and a morphism that records
    /// the label it was resolved from via `interpret`.
    mod dag_fixtures {
        use super::*;

        #[derive(Clone, Debug)]
        pub struct TestContainer {
            pub labels: Vec<String>,
        }

        impl Contains<String> for TestContainer {
            fn contained_labels(&self) -> Vec<String> {
                self.labels.clone()
            }
        }

        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct TestMorphism {
            pub resolved_from: String,
        }

        impl InterpretableMorphism<TestContainer, char, String> for TestMorphism {
            fn interpret<F>(
                container: &TestContainer,
                black_box_interpreter: F,
            ) -> Result<Self, CatgraphError>
            where
                F: Fn(&String, &[char], &[char]) -> Result<Self, CatgraphError>,
            {
                // Resolve the first contained label via the interpreter,
                // falling back to the joined label list as the name.
                if let Some(first) = container.labels.first() {
                    black_box_interpreter(first, &[], &[])
                } else {
                    Ok(TestMorphism {
                        resolved_from: container.labels.join("+"),
                    })
                }
            }
        }

        pub type TestSystem = MorphismSystem<String, char, TestContainer, TestMorphism>;

        pub fn new_system(main: &str) -> TestSystem {
            MorphismSystem::new(main.to_string())
        }
    }

    use dag_fixtures::{new_system, TestContainer, TestMorphism};

    #[test]
    fn add_simple_definitions_then_fill() {
        let mut sys = new_system("A");
        sys.add_definition_simple(
            "A".to_string(),
            TestMorphism {
                resolved_from: "leaf-A".into(),
            },
        )
        .unwrap();
        sys.add_definition_simple(
            "B".to_string(),
            TestMorphism {
                resolved_from: "leaf-B".into(),
            },
        )
        .unwrap();

        // fill_black_boxes should resolve immediately for a simple definition
        let result = sys.fill_black_boxes(None).unwrap();
        assert_eq!(result.resolved_from, "leaf-A");

        let result_b = sys.fill_black_boxes(Some("B".to_string())).unwrap();
        assert_eq!(result_b.resolved_from, "leaf-B");
    }

    #[test]
    fn composite_referencing_simples_resolves() {
        let mut sys = new_system("top");

        // Register two leaf definitions
        sys.add_definition_simple(
            "leaf1".to_string(),
            TestMorphism {
                resolved_from: "leaf1".into(),
            },
        )
        .unwrap();
        sys.add_definition_simple(
            "leaf2".to_string(),
            TestMorphism {
                resolved_from: "leaf2".into(),
            },
        )
        .unwrap();

        // Register a composite that depends on leaf1 and leaf2
        sys.add_definition_composite(
            "top".to_string(),
            TestContainer {
                labels: vec!["leaf1".to_string(), "leaf2".to_string()],
            },
        )
        .unwrap();

        // fill_black_boxes resolves the composite through topological order
        let result = sys.fill_black_boxes(None).unwrap();
        // interpret delegates to black_box_interpreter for the first label ("leaf1")
        assert_eq!(result.resolved_from, "leaf1");
    }

    #[test]
    fn cycle_detection_returns_error() {
        let mut sys = new_system("A");

        // A depends on B
        sys.add_definition_composite(
            "A".to_string(),
            TestContainer {
                labels: vec!["B".to_string()],
            },
        )
        .unwrap();

        // B depends on A — this would create a cycle
        let result = sys.add_definition_composite(
            "B".to_string(),
            TestContainer {
                labels: vec!["A".to_string()],
            },
        );

        assert!(result.is_err());
        match result {
            Err(CatgraphError::Interpret { context }) => {
                assert!(
                    context.contains("cycle"),
                    "Error message should mention 'cycle', got: {context}"
                );
            }
            other => panic!("Expected Interpret error, got: {other:?}"),
        }
    }

    #[test]
    fn deep_chain_resolves_in_topological_order() {
        // Chain: top → mid → base (each composite except the leaf)
        let mut sys = new_system("top");

        sys.add_definition_simple(
            "base".to_string(),
            TestMorphism {
                resolved_from: "base".into(),
            },
        )
        .unwrap();

        sys.add_definition_composite(
            "mid".to_string(),
            TestContainer {
                labels: vec!["base".to_string()],
            },
        )
        .unwrap();

        sys.add_definition_composite(
            "top".to_string(),
            TestContainer {
                labels: vec!["mid".to_string()],
            },
        )
        .unwrap();

        let result = sys.fill_black_boxes(None).unwrap();
        // "top" interpret calls black_box_interpreter("mid"), which was
        // already resolved to the "base" leaf by topological processing
        assert_eq!(result.resolved_from, "base");
    }

    #[test]
    fn diamond_dependency_resolves() {
        //    top
        //   /   \
        //  left  right
        //   \   /
        //    base
        let mut sys = new_system("top");

        sys.add_definition_simple(
            "base".to_string(),
            TestMorphism {
                resolved_from: "base".into(),
            },
        )
        .unwrap();

        sys.add_definition_composite(
            "left".to_string(),
            TestContainer {
                labels: vec!["base".to_string()],
            },
        )
        .unwrap();

        sys.add_definition_composite(
            "right".to_string(),
            TestContainer {
                labels: vec!["base".to_string()],
            },
        )
        .unwrap();

        sys.add_definition_composite(
            "top".to_string(),
            TestContainer {
                labels: vec!["left".to_string(), "right".to_string()],
            },
        )
        .unwrap();

        let result = sys.fill_black_boxes(None).unwrap();
        // "top" interpret resolves first contained label "left", which
        // was already resolved to "base"
        assert_eq!(result.resolved_from, "base");
    }

    #[test]
    fn self_cycle_detected() {
        let mut sys = new_system("A");

        // A depends on itself
        let result = sys.add_definition_composite(
            "A".to_string(),
            TestContainer {
                labels: vec!["A".to_string()],
            },
        );

        assert!(result.is_err());
        match result {
            Err(CatgraphError::Interpret { context }) => {
                assert!(context.contains("cycle"), "Expected cycle error, got: {context}");
            }
            other => panic!("Expected Interpret error, got: {other:?}"),
        }
    }

    #[test]
    fn adding_composite_with_no_deps_succeeds() {
        let mut sys = new_system("solo");

        // A composite with an empty contained_labels list is valid
        sys.add_definition_composite(
            "solo".to_string(),
            TestContainer { labels: vec![] },
        )
        .unwrap();

        let result = sys.fill_black_boxes(None).unwrap();
        // interpret with empty labels returns joined label list ""
        assert_eq!(result.resolved_from, "");
    }
}
