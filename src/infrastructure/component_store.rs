//! Component store for managing person component data

use async_trait::async_trait;
use cim_domain::{DomainError, DomainResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::aggregate::{PersonId, ComponentType};
use crate::components::data::{
    ComponentDataTrait,
    ComponentInstanceId,
    ComponentInstance,
    ComponentData, ContactData, ProfessionalData, SocialData,
    EmailComponentData, PhoneComponentData,
    EmploymentHistoryData, SkillsData,
    SocialMediaProfileData,
};

/// Trait for component storage
#[async_trait]
pub trait ComponentStore: Send + Sync {
    /// Store a component instance
    async fn store_component<T: ComponentDataTrait + Serialize + Send + Sync>(
        &self,
        component: ComponentInstance<T>,
    ) -> DomainResult<ComponentInstanceId>;
    
    /// Retrieve a component by ID
    async fn get_component<T: ComponentDataTrait + for<'de> Deserialize<'de> + Send + Sync>(
        &self,
        id: ComponentInstanceId,
    ) -> DomainResult<Option<ComponentInstance<T>>>;
    
    /// Get all components of a type for a person
    async fn get_components_by_type(
        &self,
        person_id: PersonId,
        component_type: ComponentType,
    ) -> DomainResult<Vec<serde_json::Value>>;
    
    /// Update a component
    async fn update_component<T: ComponentDataTrait + Serialize + Send + Sync>(
        &self,
        component: ComponentInstance<T>,
    ) -> DomainResult<()>;
    
    /// Delete a component
    async fn delete_component(&self, id: ComponentInstanceId) -> DomainResult<()>;
    
    /// Get all component IDs for a person
    async fn get_person_component_ids(
        &self,
        person_id: PersonId,
    ) -> DomainResult<Vec<ComponentInstanceId>>;
}

/// In-memory implementation of component store
pub struct InMemoryComponentStore {
    // Store components as JSON for flexibility
    components: Arc<RwLock<HashMap<ComponentInstanceId, StoredComponent>>>,
    // Index by person ID for efficient queries
    person_index: Arc<RwLock<HashMap<PersonId, Vec<ComponentInstanceId>>>>,
    // Index by component type
    type_index: Arc<RwLock<HashMap<(PersonId, ComponentType), Vec<ComponentInstanceId>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredComponent {
    person_id: PersonId,
    component_type: ComponentType,
    data: serde_json::Value,
}

impl Default for InMemoryComponentStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryComponentStore {
    pub fn new() -> Self {
        Self {
            components: Arc::new(RwLock::new(HashMap::new())),
            person_index: Arc::new(RwLock::new(HashMap::new())),
            type_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Add a component (convenience method)
    pub async fn add_component(
        &self,
        person_id: PersonId,
        component_data: ComponentData,
    ) -> DomainResult<ComponentInstanceId> {
        // Convert ComponentData to appropriate ComponentInstance
        match component_data {
            ComponentData::Contact(contact) => match contact {
                ContactData::Email(data) => {
                    let instance = ComponentInstance::new(person_id, data)?;
                    self.store_component(instance).await
                }
                ContactData::Phone(data) => {
                    let instance = ComponentInstance::new(person_id, data)?;
                    self.store_component(instance).await
                }
                ContactData::Messaging(data) => {
                    let instance = ComponentInstance::new(person_id, data)?;
                    self.store_component(instance).await
                }
            },
            ComponentData::Professional(prof) => match prof {
                ProfessionalData::Employment(data) => {
                    let instance = ComponentInstance::new(person_id, data)?;
                    self.store_component(instance).await
                }
                ProfessionalData::Affiliation(data) => {
                    let instance = ComponentInstance::new(person_id, data)?;
                    self.store_component(instance).await
                }
                ProfessionalData::Project(data) => {
                    let instance = ComponentInstance::new(person_id, data)?;
                    self.store_component(instance).await
                }
                ProfessionalData::Skills(data) => {
                    let instance = ComponentInstance::new(person_id, data)?;
                    self.store_component(instance).await
                }
            },
            ComponentData::Social(social) => match social {
                SocialData::SocialMedia(data) => {
                    let instance = ComponentInstance::new(person_id, data)?;
                    self.store_component(instance).await
                }
                SocialData::Website(data) => {
                    let instance = ComponentInstance::new(person_id, data)?;
                    self.store_component(instance).await
                }
                SocialData::ProfessionalNetwork(data) => {
                    let instance = ComponentInstance::new(person_id, data)?;
                    self.store_component(instance).await
                }
                SocialData::Relationship(_data) => {
                    // Handle relationship data
                    Err(DomainError::ValidationError("Relationship components not yet implemented".to_string()))
                }
            },
            ComponentData::Location(_data) => {
                // Handle location data
                Err(DomainError::ValidationError("Location components not yet implemented".to_string()))
            }
            ComponentData::Preferences(_data) => {
                // Handle preferences data
                Err(DomainError::ValidationError("Preferences components not yet implemented".to_string()))
            }
        }
    }
    
    /// Get all components for a person
    pub async fn get_components(
        &self,
        person_id: PersonId,
    ) -> DomainResult<Vec<ComponentData>> {
        let person_index = self.person_index.read().await;
        let components = self.components.read().await;
        
        let component_ids = person_index
            .get(&person_id)
            .cloned()
            .unwrap_or_default();
        
        let mut results = Vec::new();
        for id in component_ids {
            if let Some(stored) = components.get(&id) {
                // Deserialize to ComponentData
                if let Ok(data) = serde_json::from_value::<ComponentData>(stored.data.clone()) {
                    results.push(data);
                }
            }
        }
        
        Ok(results)
    }
    
    /// Remove a component
    pub async fn remove_component(
        &self,
        _person_id: PersonId,
        component_id: ComponentInstanceId,
    ) -> DomainResult<()> {
        self.delete_component(component_id).await
    }
    
    /// Update a component
    pub async fn update_component(
        &self,
        person_id: PersonId,
        component_id: ComponentInstanceId,
        component_data: ComponentData,
    ) -> DomainResult<()> {
        // Get the existing component to preserve metadata
        let components = self.components.read().await;
        if let Some(stored) = components.get(&component_id) {
            // Deserialize to get the metadata
            let existing_value = &stored.data;
            
            // Update based on component type
            match component_data {
                ComponentData::Contact(contact) => match contact {
                    ContactData::Email(data) => {
                        // Create new instance with same ID
                        let mut instance = ComponentInstance::new(person_id, data)?;
                        instance.id = component_id;
                        // Preserve metadata if possible
                        if let Ok(existing) = serde_json::from_value::<ComponentInstance<EmailComponentData>>(existing_value.clone()) {
                            instance.metadata = existing.metadata;
                            instance.metadata.updated_at = chrono::Utc::now();
                        }
                        drop(components);
                        ComponentStore::update_component(self, instance).await
                    }
                    ContactData::Phone(data) => {
                        let mut instance = ComponentInstance::new(person_id, data)?;
                        instance.id = component_id;
                        if let Ok(existing) = serde_json::from_value::<ComponentInstance<PhoneComponentData>>(existing_value.clone()) {
                            instance.metadata = existing.metadata;
                            instance.metadata.updated_at = chrono::Utc::now();
                        }
                        drop(components);
                        ComponentStore::update_component(self, instance).await
                    }
                    _ => Err(DomainError::ValidationError("Component type not supported for update".to_string()))
                },
                ComponentData::Professional(prof) => match prof {
                    ProfessionalData::Skills(data) => {
                        let mut instance = ComponentInstance::new(person_id, data)?;
                        instance.id = component_id;
                        if let Ok(existing) = serde_json::from_value::<ComponentInstance<SkillsData>>(existing_value.clone()) {
                            instance.metadata = existing.metadata;
                            instance.metadata.updated_at = chrono::Utc::now();
                        }
                        drop(components);
                        ComponentStore::update_component(self, instance).await
                    }
                    ProfessionalData::Employment(data) => {
                        let mut instance = ComponentInstance::new(person_id, data)?;
                        instance.id = component_id;
                        if let Ok(existing) = serde_json::from_value::<ComponentInstance<EmploymentHistoryData>>(existing_value.clone()) {
                            instance.metadata = existing.metadata;
                            instance.metadata.updated_at = chrono::Utc::now();
                        }
                        drop(components);
                        ComponentStore::update_component(self, instance).await
                    }
                    _ => Err(DomainError::ValidationError("Component type not supported for update".to_string()))
                },
                ComponentData::Social(social) => match social {
                    SocialData::SocialMedia(data) => {
                        let mut instance = ComponentInstance::new(person_id, data)?;
                        instance.id = component_id;
                        if let Ok(existing) = serde_json::from_value::<ComponentInstance<SocialMediaProfileData>>(existing_value.clone()) {
                            instance.metadata = existing.metadata;
                            instance.metadata.updated_at = chrono::Utc::now();
                        }
                        drop(components);
                        ComponentStore::update_component(self, instance).await
                    }
                    _ => Err(DomainError::ValidationError("Component type not supported for update".to_string()))
                },
                _ => Err(DomainError::ValidationError("Component type not supported for update".to_string()))
            }
        } else {
            Err(DomainError::generic(format!("Component {component_id} not found")))
        }
    }
}

#[async_trait]
impl ComponentStore for InMemoryComponentStore {
    async fn store_component<T: ComponentDataTrait + Serialize + Send + Sync>(
        &self,
        component: ComponentInstance<T>,
    ) -> DomainResult<ComponentInstanceId> {
        let id = component.id;
        let person_id = component.person_id;
        let component_type = component.data.component_type();
        
        let stored = StoredComponent {
            person_id,
            component_type: component_type.clone(),
            data: serde_json::to_value(&component)
                .map_err(|e| DomainError::SerializationError(e.to_string()))?,
        };

        // Store component
        self.components.write().await.insert(id, stored);

        // Update person index
        self.person_index
            .write()
            .await
            .entry(person_id)
            .or_insert_with(Vec::new)
            .push(id);

        // Update type index
        self.type_index
            .write()
            .await
            .entry((person_id, component_type))
            .or_insert_with(Vec::new)
            .push(id);
        
        Ok(id)
    }
    
    async fn get_component<T: ComponentDataTrait + for<'de> Deserialize<'de> + Send + Sync>(
        &self,
        id: ComponentInstanceId,
    ) -> DomainResult<Option<ComponentInstance<T>>> {
        let components = self.components.read().await;
        
        match components.get(&id) {
            Some(stored) => {
                let component: ComponentInstance<T> = serde_json::from_value(stored.data.clone())
                    .map_err(|e| DomainError::SerializationError(e.to_string()))?;
                Ok(Some(component))
            }
            None => Ok(None),
        }
    }
    
    async fn get_components_by_type(
        &self,
        person_id: PersonId,
        component_type: ComponentType,
    ) -> DomainResult<Vec<serde_json::Value>> {
        let type_index = self.type_index.read().await;
        let components = self.components.read().await;
        
        let component_ids = type_index
            .get(&(person_id, component_type))
            .cloned()
            .unwrap_or_default();
        
        let mut results = Vec::new();
        for id in component_ids {
            if let Some(stored) = components.get(&id) {
                results.push(stored.data.clone());
            }
        }
        
        Ok(results)
    }
    
    async fn update_component<T: ComponentDataTrait + Serialize + Send + Sync>(
        &self,
        component: ComponentInstance<T>,
    ) -> DomainResult<()> {
        let id = component.id;
        let person_id = component.person_id;
        let component_type = component.data.component_type();
        
        let stored = StoredComponent {
            person_id,
            component_type,
            data: serde_json::to_value(&component)
                .map_err(|e| DomainError::SerializationError(e.to_string()))?,
        };
        
        let mut components = self.components.write().await;
        
        if !components.contains_key(&id) {
            return Err(DomainError::generic(format!("Component {id} not found")));
        }
        
        components.insert(id, stored);
        Ok(())
    }
    
    async fn delete_component(&self, id: ComponentInstanceId) -> DomainResult<()> {
        let mut components = self.components.write().await;
        
        if let Some(stored) = components.remove(&id) {
            // Remove from person index
            let mut person_index = self.person_index.write().await;
            if let Some(ids) = person_index.get_mut(&stored.person_id) {
                ids.retain(|&comp_id| comp_id != id);
            }
            
            // Remove from type index
            let mut type_index = self.type_index.write().await;
            if let Some(ids) = type_index.get_mut(&(stored.person_id, stored.component_type)) {
                ids.retain(|&comp_id| comp_id != id);
            }
            
            Ok(())
        } else {
            Err(DomainError::generic(format!("Component {id} not found")))
        }
    }
    
    async fn get_person_component_ids(
        &self,
        person_id: PersonId,
    ) -> DomainResult<Vec<ComponentInstanceId>> {
        let person_index = self.person_index.read().await;
        Ok(person_index.get(&person_id).cloned().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::EmailAddress;
    
    #[tokio::test]
    async fn test_store_and_retrieve_component() {
        let store = InMemoryComponentStore::new();
        let person_id = PersonId::new();
        
        // Create email component
        let email_data = EmailComponentData {
            email: EmailAddress::new("test@example.com".to_string()).expect("Valid email"),
            email_type: crate::components::data::EmailType::Personal,
            is_preferred_contact: true,
            can_receive_notifications: true,
            can_receive_marketing: false,
        };
        
        let component = ComponentInstance::new(person_id, email_data).unwrap();
        let id = component.id;
        
        // Store component
        let stored_id = store.store_component(component).await.unwrap();
        assert_eq!(stored_id, id);
        
        // Retrieve component
        let retrieved: Option<ComponentInstance<EmailComponentData>> = 
            store.get_component(id).await.unwrap();
        
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, id);
        assert_eq!(retrieved.person_id, person_id);
    }
    
    #[tokio::test]
    async fn test_get_components_by_type() {
        let store = InMemoryComponentStore::new();
        let person_id = PersonId::new();
        
        // Store multiple email components
        for i in 0..3 {
            let email_data = EmailComponentData {
                email: EmailAddress::new(format!("test{i}@example.com")).expect("Valid email"),
                email_type: crate::components::data::EmailType::Personal,
                is_preferred_contact: i == 0,
                can_receive_notifications: true,
                can_receive_marketing: false,
            };
            
            let component = ComponentInstance::new(person_id, email_data).unwrap();
            store.store_component(component).await.unwrap();
        }
        
        // Get all email components
        let components = store
            .get_components_by_type(person_id, ComponentType::EmailAddress)
            .await
            .unwrap();
        
        assert_eq!(components.len(), 3);
    }
} 