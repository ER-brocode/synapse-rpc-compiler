use std::collections::{HashMap, HashSet};
use crate::error::{Result, SynapseError};
use crate::parser::ast::*;

pub struct Validator<'a> {
    schema: &'a Schema,
    defined_types: HashSet<String>,
}

impl<'a> Validator<'a> {
    pub fn new(schema: &'a Schema) -> Self {
        let mut defined_types = HashSet::new();
        for s in &schema.structs {
            defined_types.insert(s.name.clone());
        }
        for e in &schema.enums {
            defined_types.insert(e.name.clone());
        }

        Self {
            schema,
            defined_types,
        }
    }

    pub fn validate(&self) -> Result<()> {
        self.check_unique_declarations()?;
        self.validate_structs()?;
        self.validate_enums()?;
        self.validate_services()?;
        self.check_recursive_types()?;
        Ok(())
    }

    fn check_unique_declarations(&self) -> Result<()> {
        let mut seen = HashSet::new();

        for s in &self.schema.structs {
            if !seen.insert(s.name.clone()) {
                return Err(SynapseError::ValidationError {
                    message: format!("Duplicate struct name '{}'", s.name),
                    context: Some(s.name.clone()),
                });
            }
        }

        for e in &self.schema.enums {
            if !seen.insert(e.name.clone()) {
                return Err(SynapseError::ValidationError {
                    message: format!("Duplicate type name '{}' between struct and enum", e.name),
                    context: Some(e.name.clone()),
                });
            }
        }

        let mut service_names = HashSet::new();
        for s in &self.schema.services {
            if !service_names.insert(s.name.clone()) {
                return Err(SynapseError::ValidationError {
                    message: format!("Duplicate service name '{}'", s.name),
                    context: Some(s.name.clone()),
                });
            }
        }

        Ok(())
    }

    fn validate_structs(&self) -> Result<()> {
        for s in &self.schema.structs {
            let mut field_ids = HashSet::new();
            let mut field_names = HashSet::new();

            for f in &s.fields {
                if f.id == 0 {
                    return Err(SynapseError::ValidationError {
                        message: format!("Struct '{}' field '{}' has invalid field id 0 (field IDs must be >= 1)", s.name, f.name),
                        context: Some(s.name.clone()),
                    });
                }

                if !field_ids.insert(f.id) {
                    return Err(SynapseError::ValidationError {
                        message: format!("Struct '{}' contains duplicate field ID {}", s.name, f.id),
                        context: Some(s.name.clone()),
                    });
                }

                if !field_names.insert(f.name.clone()) {
                    return Err(SynapseError::ValidationError {
                        message: format!("Struct '{}' contains duplicate field name '{}'", s.name, f.name),
                        context: Some(s.name.clone()),
                    });
                }

                self.validate_field_type(&f.field_type, &format!("struct {}.{}", s.name, f.name))?;
            }
        }
        Ok(())
    }

    fn validate_enums(&self) -> Result<()> {
        for e in &self.schema.enums {
            let mut variant_names = HashSet::new();
            let mut variant_values = HashSet::new();

            if e.variants.is_empty() {
                return Err(SynapseError::ValidationError {
                    message: format!("Enum '{}' must have at least one variant", e.name),
                    context: Some(e.name.clone()),
                });
            }

            for v in &e.variants {
                if !variant_names.insert(v.name.clone()) {
                    return Err(SynapseError::ValidationError {
                        message: format!("Enum '{}' has duplicate variant name '{}'", e.name, v.name),
                        context: Some(e.name.clone()),
                    });
                }
                if !variant_values.insert(v.value) {
                    return Err(SynapseError::ValidationError {
                        message: format!("Enum '{}' has duplicate variant value {}", e.name, v.value),
                        context: Some(e.name.clone()),
                    });
                }
            }
        }
        Ok(())
    }

    fn validate_services(&self) -> Result<()> {
        for s in &self.schema.services {
            let mut method_names = HashSet::new();

            for m in &s.methods {
                if !method_names.insert(m.name.clone()) {
                    return Err(SynapseError::ValidationError {
                        message: format!("Service '{}' has duplicate method name '{}'", s.name, m.name),
                        context: Some(s.name.clone()),
                    });
                }

                self.validate_field_type(&m.request_type, &format!("service {}.{} request", s.name, m.name))?;
                self.validate_field_type(&m.response_type, &format!("service {}.{} response", s.name, m.name))?;
            }
        }
        Ok(())
    }

    fn validate_field_type(&self, ty: &FieldType, context: &str) -> Result<()> {
        match ty {
            FieldType::Primitive(_) => Ok(()),
            FieldType::List(inner) => self.validate_field_type(inner, context),
            FieldType::Map(k, v) => {
                match &**k {
                    FieldType::Primitive(_) => {}
                    _ => {
                        return Err(SynapseError::ValidationError {
                            message: format!("Map key in {} must be a primitive type", context),
                            context: Some(context.to_string()),
                        });
                    }
                }
                self.validate_field_type(v, context)
            }
            FieldType::Option(inner) => self.validate_field_type(inner, context),
            FieldType::Result { ok, err } => {
                self.validate_field_type(ok, context)?;
                self.validate_field_type(err, context)
            }
            FieldType::Custom(name) => {
                if !self.defined_types.contains(name) {
                    return Err(SynapseError::ValidationError {
                        message: format!("Undefined type '{}' referenced in {}", name, context),
                        context: Some(context.to_string()),
                    });
                }
                Ok(())
            }
        }
    }

    fn check_recursive_types(&self) -> Result<()> {
        let mut struct_map = HashMap::new();
        for s in &self.schema.structs {
            struct_map.insert(s.name.as_str(), s);
        }

        for s in &self.schema.structs {
            let mut visited = HashSet::new();
            self.check_cycle(s.name.as_str(), &struct_map, &mut visited)?;
        }

        Ok(())
    }

    fn check_cycle<'b>(
        &self,
        current: &'b str,
        struct_map: &HashMap<&'b str, &'b StructDef>,
        visited: &mut HashSet<&'b str>,
    ) -> Result<()> {
        if !visited.insert(current) {
            return Err(SynapseError::ValidationError {
                message: format!("Direct recursive reference detected in struct '{}' without indirection (e.g. option or list)", current),
                context: Some(current.to_string()),
            });
        }

        if let Some(s) = struct_map.get(current) {
            for f in &s.fields {
                if let FieldType::Custom(ref nested) = f.field_type {
                    if struct_map.contains_key(nested.as_str()) {
                        self.check_cycle(nested.as_str(), struct_map, visited)?;
                    }
                }
            }
        }

        visited.remove(current);
        Ok(())
    }
}
