use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use anyhow::Result;
use crate::ai_operations::{AbstractBlockSpec, BlockType, BehaviorSpec};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignPattern {
    pub name: String,
    pub description: String,
    pub template: String,
    pub placeholders: Vec<String>,
    pub constraints: Vec<String>,
    pub language_variants: HashMap<String, LanguageVariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageVariant {
    pub template: String,
    pub imports: Vec<String>,
    pub dependencies: Vec<String>,
}

pub struct PatternLibrary {
    patterns: HashMap<String, DesignPattern>,
}

impl PatternLibrary {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // Initialize with basic patterns
        patterns.insert("function".to_string(), Self::create_function_pattern());
        patterns.insert("class".to_string(), Self::create_class_pattern());
        patterns.insert("async_function".to_string(), Self::create_async_function_pattern());
        patterns.insert("singleton".to_string(), Self::create_singleton_pattern());
        patterns.insert("factory".to_string(), Self::create_factory_pattern());
        patterns.insert("builder".to_string(), Self::create_builder_pattern());
        patterns.insert("observer".to_string(), Self::create_observer_pattern());
        patterns.insert("strategy".to_string(), Self::create_strategy_pattern());
        patterns.insert("facade".to_string(), Self::create_facade_pattern());
        patterns.insert("default".to_string(), Self::create_default_pattern());
        
        Self { patterns }
    }

    pub fn select_pattern(&self, spec: &AbstractBlockSpec) -> Result<DesignPattern> {
        // Match abstraction to appropriate design pattern
        let pattern_name = match (&spec.block_type, &spec.behaviors) {
            (BlockType::Class, behaviors) if self.is_singleton(behaviors) => "singleton",
            (BlockType::Class, behaviors) if self.is_factory(behaviors) => "factory",
            (BlockType::Class, behaviors) if self.is_builder(behaviors) => "builder",
            (BlockType::Class, behaviors) if self.is_observer(behaviors) => "observer",
            (BlockType::Class, behaviors) if self.is_strategy(behaviors) => "strategy",
            (BlockType::Class, behaviors) if self.is_facade(behaviors) => "facade",
            (BlockType::Function, _) if spec.properties.is_async => "async_function",
            (BlockType::Function, _) => "function",
            (BlockType::Class, _) => "class",
            _ => "default",
        };

        self.patterns
            .get(pattern_name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Pattern not found: {}", pattern_name))
    }

    pub fn get_pattern(&self, name: &str) -> Option<&DesignPattern> {
        self.patterns.get(name)
    }

    pub fn add_pattern(&mut self, name: String, pattern: DesignPattern) {
        self.patterns.insert(name, pattern);
    }

    // Pattern detection methods
    fn is_singleton(&self, behaviors: &[BehaviorSpec]) -> bool {
        behaviors.iter().any(|b| {
            b.name.contains("getInstance") || 
            b.description.to_lowercase().contains("single instance")
        })
    }

    fn is_factory(&self, behaviors: &[BehaviorSpec]) -> bool {
        behaviors.iter().any(|b| {
            b.name.contains("create") || b.name.contains("make") ||
            b.description.to_lowercase().contains("factory") ||
            b.description.to_lowercase().contains("create object")
        })
    }

    fn is_builder(&self, behaviors: &[BehaviorSpec]) -> bool {
        behaviors.iter().any(|b| {
            b.name.contains("build") || b.name.contains("with") ||
            b.description.to_lowercase().contains("builder") ||
            b.description.to_lowercase().contains("step by step")
        })
    }

    fn is_observer(&self, behaviors: &[BehaviorSpec]) -> bool {
        behaviors.iter().any(|b| {
            b.name.contains("notify") || b.name.contains("subscribe") ||
            b.name.contains("observe") || b.name.contains("listen") ||
            b.description.to_lowercase().contains("observer") ||
            b.description.to_lowercase().contains("event")
        })
    }

    fn is_strategy(&self, behaviors: &[BehaviorSpec]) -> bool {
        behaviors.iter().any(|b| {
            b.name.contains("execute") || b.name.contains("algorithm") ||
            b.description.to_lowercase().contains("strategy") ||
            b.description.to_lowercase().contains("algorithm")
        })
    }

    fn is_facade(&self, behaviors: &[BehaviorSpec]) -> bool {
        behaviors.iter().any(|b| {
            b.description.to_lowercase().contains("facade") ||
            b.description.to_lowercase().contains("simplified interface") ||
            b.description.to_lowercase().contains("unified interface")
        })
    }

    // Pattern creation methods
    fn create_function_pattern() -> DesignPattern {
        let mut language_variants = HashMap::new();
        
        language_variants.insert("python".to_string(), LanguageVariant {
            template: r#"def {{function_name}}({{parameters}}):
    """{{description}}"""
    {{body}}
    return {{return_value}}"#.to_string(),
            imports: vec![],
            dependencies: vec![],
        });

        language_variants.insert("typescript".to_string(), LanguageVariant {
            template: r#"function {{function_name}}({{parameters}}): {{return_type}} {
    // {{description}}
    {{body}}
    return {{return_value}};
}"#.to_string(),
            imports: vec![],
            dependencies: vec![],
        });

        language_variants.insert("rust".to_string(), LanguageVariant {
            template: r#"fn {{function_name}}({{parameters}}) -> {{return_type}} {
    // {{description}}
    {{body}}
    {{return_value}}
}"#.to_string(),
            imports: vec![],
            dependencies: vec![],
        });

        DesignPattern {
            name: "function".to_string(),
            description: "Basic function pattern".to_string(),
            template: "{{function_name}}({{parameters}})".to_string(),
            placeholders: vec![
                "function_name".to_string(),
                "parameters".to_string(),
                "description".to_string(),
                "body".to_string(),
                "return_value".to_string(),
                "return_type".to_string(),
            ],
            constraints: vec![],
            language_variants,
        }
    }

    fn create_async_function_pattern() -> DesignPattern {
        let mut language_variants = HashMap::new();
        
        language_variants.insert("python".to_string(), LanguageVariant {
            template: r#"async def {{function_name}}({{parameters}}):
    """{{description}}"""
    {{body}}
    return {{return_value}}"#.to_string(),
            imports: vec!["import asyncio".to_string()],
            dependencies: vec![],
        });

        language_variants.insert("typescript".to_string(), LanguageVariant {
            template: r#"async function {{function_name}}({{parameters}}): Promise<{{return_type}}> {
    // {{description}}
    {{body}}
    return {{return_value}};
}"#.to_string(),
            imports: vec![],
            dependencies: vec![],
        });

        language_variants.insert("rust".to_string(), LanguageVariant {
            template: r#"async fn {{function_name}}({{parameters}}) -> {{return_type}} {
    // {{description}}
    {{body}}
    {{return_value}}
}"#.to_string(),
            imports: vec!["use tokio".to_string()],
            dependencies: vec!["tokio".to_string()],
        });

        DesignPattern {
            name: "async_function".to_string(),
            description: "Asynchronous function pattern".to_string(),
            template: "async {{function_name}}({{parameters}})".to_string(),
            placeholders: vec![
                "function_name".to_string(),
                "parameters".to_string(),
                "description".to_string(),
                "body".to_string(),
                "return_value".to_string(),
                "return_type".to_string(),
            ],
            constraints: vec!["requires_async_runtime".to_string()],
            language_variants,
        }
    }

    fn create_class_pattern() -> DesignPattern {
        let mut language_variants = HashMap::new();
        
        language_variants.insert("python".to_string(), LanguageVariant {
            template: r#"class {{class_name}}:
    """{{description}}"""
    
    def __init__(self{{init_parameters}}):
        {{init_body}}
    
    {{methods}}"#.to_string(),
            imports: vec![],
            dependencies: vec![],
        });

        language_variants.insert("typescript".to_string(), LanguageVariant {
            template: r#"class {{class_name}} {
    // {{description}}
    
    constructor({{init_parameters}}) {
        {{init_body}}
    }
    
    {{methods}}
}"#.to_string(),
            imports: vec![],
            dependencies: vec![],
        });

        language_variants.insert("rust".to_string(), LanguageVariant {
            template: r#"/// {{description}}
struct {{class_name}} {
    {{fields}}
}

impl {{class_name}} {
    fn new({{init_parameters}}) -> Self {
        {{init_body}}
    }
    
    {{methods}}
}"#.to_string(),
            imports: vec![],
            dependencies: vec![],
        });

        DesignPattern {
            name: "class".to_string(),
            description: "Basic class pattern".to_string(),
            template: "class {{class_name}}".to_string(),
            placeholders: vec![
                "class_name".to_string(),
                "description".to_string(),
                "init_parameters".to_string(),
                "init_body".to_string(),
                "methods".to_string(),
                "fields".to_string(),
            ],
            constraints: vec![],
            language_variants,
        }
    }

    fn create_singleton_pattern() -> DesignPattern {
        let mut language_variants = HashMap::new();
        
        language_variants.insert("python".to_string(), LanguageVariant {
            template: r#"class {{class_name}}:
    """{{description}} - Singleton Pattern"""
    _instance = None
    _lock = threading.Lock()
    
    def __new__(cls):
        if cls._instance is None:
            with cls._lock:
                if cls._instance is None:
                    cls._instance = super().__new__(cls)
        return cls._instance
    
    def __init__(self):
        if not hasattr(self, '_initialized'):
            {{init_body}}
            self._initialized = True
    
    {{methods}}"#.to_string(),
            imports: vec!["import threading".to_string()],
            dependencies: vec![],
        });

        DesignPattern {
            name: "singleton".to_string(),
            description: "Singleton design pattern".to_string(),
            template: "singleton {{class_name}}".to_string(),
            placeholders: vec![
                "class_name".to_string(),
                "description".to_string(),
                "init_body".to_string(),
                "methods".to_string(),
            ],
            constraints: vec!["thread_safe".to_string()],
            language_variants,
        }
    }

    fn create_factory_pattern() -> DesignPattern {
        let mut language_variants = HashMap::new();
        
        language_variants.insert("python".to_string(), LanguageVariant {
            template: r#"class {{class_name}}:
    """{{description}} - Factory Pattern"""
    
    @staticmethod
    def create(product_type: str, **kwargs):
        """Factory method to create products"""
        {{factory_logic}}
    
    {{methods}}"#.to_string(),
            imports: vec![],
            dependencies: vec![],
        });

        DesignPattern {
            name: "factory".to_string(),
            description: "Factory design pattern".to_string(),
            template: "factory {{class_name}}".to_string(),
            placeholders: vec![
                "class_name".to_string(),
                "description".to_string(),
                "factory_logic".to_string(),
                "methods".to_string(),
            ],
            constraints: vec![],
            language_variants,
        }
    }

    fn create_builder_pattern() -> DesignPattern {
        let mut language_variants = HashMap::new();
        
        language_variants.insert("python".to_string(), LanguageVariant {
            template: r#"class {{class_name}}Builder:
    """{{description}} - Builder Pattern"""
    
    def __init__(self):
        self._product = {{class_name}}()
    
    {{builder_methods}}
    
    def build(self) -> '{{class_name}}':
        """Build and return the final product"""
        return self._product

class {{class_name}}:
    """The product being built"""
    {{product_methods}}"#.to_string(),
            imports: vec![],
            dependencies: vec![],
        });

        DesignPattern {
            name: "builder".to_string(),
            description: "Builder design pattern".to_string(),
            template: "builder {{class_name}}".to_string(),
            placeholders: vec![
                "class_name".to_string(),
                "description".to_string(),
                "builder_methods".to_string(),
                "product_methods".to_string(),
            ],
            constraints: vec![],
            language_variants,
        }
    }

    fn create_observer_pattern() -> DesignPattern {
        let mut language_variants = HashMap::new();
        
        language_variants.insert("python".to_string(), LanguageVariant {
            template: r#"from abc import ABC, abstractmethod
from typing import List

class Observer(ABC):
    """Observer interface"""
    @abstractmethod
    def update(self, subject: 'Subject') -> None:
        pass

class {{class_name}}(Observer):
    """{{description}} - Observer Pattern"""
    
    def update(self, subject: 'Subject') -> None:
        {{update_logic}}
    
    {{methods}}

class Subject:
    """Subject that observers watch"""
    def __init__(self):
        self._observers: List[Observer] = []
    
    def attach(self, observer: Observer) -> None:
        self._observers.append(observer)
    
    def detach(self, observer: Observer) -> None:
        self._observers.remove(observer)
    
    def notify(self) -> None:
        for observer in self._observers:
            observer.update(self)"#.to_string(),
            imports: vec!["from abc import ABC, abstractmethod".to_string(), "from typing import List".to_string()],
            dependencies: vec![],
        });

        DesignPattern {
            name: "observer".to_string(),
            description: "Observer design pattern".to_string(),
            template: "observer {{class_name}}".to_string(),
            placeholders: vec![
                "class_name".to_string(),
                "description".to_string(),
                "update_logic".to_string(),
                "methods".to_string(),
            ],
            constraints: vec![],
            language_variants,
        }
    }

    fn create_strategy_pattern() -> DesignPattern {
        let mut language_variants = HashMap::new();
        
        language_variants.insert("python".to_string(), LanguageVariant {
            template: r#"from abc import ABC, abstractmethod

class Strategy(ABC):
    """Strategy interface"""
    @abstractmethod
    def execute(self, data) -> any:
        pass

class {{class_name}}(Strategy):
    """{{description}} - Strategy Pattern"""
    
    def execute(self, data) -> any:
        {{execute_logic}}
    
    {{methods}}"#.to_string(),
            imports: vec!["from abc import ABC, abstractmethod".to_string()],
            dependencies: vec![],
        });

        DesignPattern {
            name: "strategy".to_string(),
            description: "Strategy design pattern".to_string(),
            template: "strategy {{class_name}}".to_string(),
            placeholders: vec![
                "class_name".to_string(),
                "description".to_string(),
                "execute_logic".to_string(),
                "methods".to_string(),
            ],
            constraints: vec![],
            language_variants,
        }
    }

    fn create_facade_pattern() -> DesignPattern {
        let mut language_variants = HashMap::new();
        
        language_variants.insert("python".to_string(), LanguageVariant {
            template: r#"class {{class_name}}:
    """{{description}} - Facade Pattern"""
    
    def __init__(self):
        # Initialize subsystems
        {{subsystem_init}}
    
    {{facade_methods}}"#.to_string(),
            imports: vec![],
            dependencies: vec![],
        });

        DesignPattern {
            name: "facade".to_string(),
            description: "Facade design pattern".to_string(),
            template: "facade {{class_name}}".to_string(),
            placeholders: vec![
                "class_name".to_string(),
                "description".to_string(),
                "subsystem_init".to_string(),
                "facade_methods".to_string(),
            ],
            constraints: vec![],
            language_variants,
        }
    }

    fn create_default_pattern() -> DesignPattern {
        let mut language_variants = HashMap::new();
        
        language_variants.insert("python".to_string(), LanguageVariant {
            template: r#"# {{description}}
{{code}}"#.to_string(),
            imports: vec![],
            dependencies: vec![],
        });

        DesignPattern {
            name: "default".to_string(),
            description: "Default pattern for unspecified structures".to_string(),
            template: "{{code}}".to_string(),
            placeholders: vec![
                "description".to_string(),
                "code".to_string(),
            ],
            constraints: vec![],
            language_variants,
        }
    }
}

impl Default for PatternLibrary {
    fn default() -> Self {
        Self::new()
    }
}
