use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::components::CodeComponent;

/// Analyzes relationships between code components
pub struct RelationshipAnalyzer {
    analyzers: Vec<Box<dyn RelationshipDetector>>,
}

impl RelationshipAnalyzer {
    pub fn new() -> Self {
        Self {
            analyzers: vec![
                Box::new(FunctionCallDetector),
                Box::new(InheritanceDetector),
                Box::new(CompositionDetector),
                Box::new(DependencyDetector),
            ],
        }
    }

    /// Analyze relationships between components
    pub fn analyze(&self, components: &[CodeComponent]) -> Result<Vec<ComponentRelationship>> {
        let mut relationships = Vec::new();

        for analyzer in &self.analyzers {
            relationships.extend(analyzer.detect(components)?);
        }

        // Remove duplicates
        let unique_relationships = self.deduplicate_relationships(relationships);

        Ok(unique_relationships)
    }

    fn deduplicate_relationships(&self, relationships: Vec<ComponentRelationship>) -> Vec<ComponentRelationship> {
        let mut seen = HashSet::new();
        let mut unique = Vec::new();

        for rel in relationships {
            let key = format!("{}-{}-{:?}", rel.from_component, rel.to_component, rel.relationship_type);
            if seen.insert(key) {
                unique.push(rel);
            }
        }

        unique
    }
}

/// Relationship between two code components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentRelationship {
    pub from_component: String,
    pub to_component: String,
    pub relationship_type: RelationshipType,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationshipType {
    FunctionCall,
    MethodCall,
    Inheritance,
    Composition,
    Dependency,
    Import,
    TypeReference,
    Override,
}

/// Trait for detecting specific types of relationships
trait RelationshipDetector: Send + Sync {
    fn detect(&self, components: &[CodeComponent]) -> Result<Vec<ComponentRelationship>>;
}

/// Detects function call relationships
struct FunctionCallDetector;

impl RelationshipDetector for FunctionCallDetector {
    fn detect(&self, components: &[CodeComponent]) -> Result<Vec<ComponentRelationship>> {
        let mut relationships = Vec::new();
        
        // Build a map of all function names
        let function_names: HashSet<String> = components.iter()
            .filter_map(|c| match c {
                CodeComponent::FunctionSignature(sig) => Some(sig.name.clone()),
                _ => None,
            })
            .collect();

        // Look for function calls in function bodies
        for component in components {
            if let CodeComponent::FunctionBody(body) = component {
                for called_func in &body.called_functions {
                    if function_names.contains(called_func) {
                        relationships.push(ComponentRelationship {
                            from_component: "current_function".to_string(), // TODO: Track current function
                            to_component: called_func.clone(),
                            relationship_type: RelationshipType::FunctionCall,
                            metadata: HashMap::new(),
                        });
                    }
                }
            }
        }

        Ok(relationships)
    }
}

/// Detects inheritance relationships
struct InheritanceDetector;

impl RelationshipDetector for InheritanceDetector {
    fn detect(&self, components: &[CodeComponent]) -> Result<Vec<ComponentRelationship>> {
        let mut relationships = Vec::new();

        // Build a map of all class names
        let class_names: HashSet<String> = components.iter()
            .filter_map(|c| match c {
                CodeComponent::ClassDeclaration(decl) => Some(decl.name.clone()),
                _ => None,
            })
            .collect();

        // Look for inheritance relationships
        for component in components {
            if let CodeComponent::ClassDeclaration(decl) = component {
                for base_class in &decl.base_classes {
                    if class_names.contains(base_class) {
                        relationships.push(ComponentRelationship {
                            from_component: decl.name.clone(),
                            to_component: base_class.clone(),
                            relationship_type: RelationshipType::Inheritance,
                            metadata: HashMap::new(),
                        });
                    }
                }
            }
        }

        Ok(relationships)
    }
}

/// Detects composition relationships
struct CompositionDetector;

impl RelationshipDetector for CompositionDetector {
    fn detect(&self, components: &[CodeComponent]) -> Result<Vec<ComponentRelationship>> {
        let mut relationships = Vec::new();

        // Build a map of all class names
        let class_names: HashSet<String> = components.iter()
            .filter_map(|c| match c {
                CodeComponent::ClassDeclaration(decl) => Some(decl.name.clone()),
                _ => None,
            })
            .collect();

        // Look for class attributes that reference other classes
        for component in components {
            if let CodeComponent::ClassBody(body) = component {
                for attribute in &body.attributes {
                    if let Some(type_ann) = &attribute.type_annotation {
                        if class_names.contains(&type_ann.base_type) {
                            relationships.push(ComponentRelationship {
                                from_component: "current_class".to_string(), // TODO: Track current class
                                to_component: type_ann.base_type.clone(),
                                relationship_type: RelationshipType::Composition,
                                metadata: HashMap::new(),
                            });
                        }
                    }
                }
            }
        }

        Ok(relationships)
    }
}

/// Detects general dependency relationships
struct DependencyDetector;

impl RelationshipDetector for DependencyDetector {
    fn detect(&self, components: &[CodeComponent]) -> Result<Vec<ComponentRelationship>> {
        let mut relationships = Vec::new();

        // Look for import statements
        for component in components {
            if let CodeComponent::Import(import) = component {
                for imported_name in &import.imported_names {
                    relationships.push(ComponentRelationship {
                        from_component: "module".to_string(), // TODO: Track current module
                        to_component: imported_name.original.clone(),
                        relationship_type: RelationshipType::Import,
                        metadata: {
                            let mut meta = HashMap::new();
                            meta.insert("module_path".to_string(), serde_json::json!(import.module_path));
                            meta.insert("is_relative".to_string(), serde_json::json!(import.is_relative));
                            meta
                        },
                    });
                }
            }
        }

        Ok(relationships)
    }
}

impl ComponentRelationship {
    pub fn new(from: String, to: String, rel_type: RelationshipType) -> Self {
        Self {
            from_component: from,
            to_component: to,
            relationship_type: rel_type,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Check if this is a local relationship (within the same module)
    pub fn is_local(&self) -> bool {
        match self.relationship_type {
            RelationshipType::Import => false,
            _ => !self.to_component.contains('.') && !self.to_component.contains('/')
        }
    }

    /// Get the strength of this relationship (for dependency analysis)
    pub fn strength(&self) -> u32 {
        match self.relationship_type {
            RelationshipType::Inheritance => 10,
            RelationshipType::Composition => 8,
            RelationshipType::Override => 7,
            RelationshipType::MethodCall => 5,
            RelationshipType::FunctionCall => 5,
            RelationshipType::TypeReference => 3,
            RelationshipType::Import => 2,
            RelationshipType::Dependency => 1,
        }
    }
}
