//! Person network projection for relationship analytics

use super::PersonProjection;
use crate::aggregate::PersonId;
use crate::events::*;
use crate::components::data::{ComponentData, SocialData};
use cim_domain::DomainResult;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Relationship types in the network
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RelationshipType {
    Family,
    Friend,
    Colleague,
    Manager,
    Report,
    Mentor,
    Mentee,
    BusinessPartner,
    Other(String),
}

/// A relationship between two people
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonRelationship {
    pub from_person: PersonId,
    pub to_person: PersonId,
    pub relationship_type: RelationshipType,
    pub strength: f32, // 0.0 to 1.0
    pub established_at: DateTime<Utc>,
    pub last_interaction: Option<DateTime<Utc>>,
    pub interaction_count: usize,
}

/// Network statistics for a person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub person_id: PersonId,
    pub total_connections: usize,
    pub connections_by_type: HashMap<RelationshipType, usize>,
    pub average_relationship_strength: f32,
    pub network_reach: usize, // 2nd degree connections
    pub clustering_coefficient: f32, // How connected their connections are
}

/// Projection that maintains person relationship networks
pub struct PersonNetworkProjection {
    relationships: Arc<RwLock<HashMap<(PersonId, PersonId), PersonRelationship>>>,
    adjacency_list: Arc<RwLock<HashMap<PersonId, HashSet<PersonId>>>>,
    reverse_adjacency: Arc<RwLock<HashMap<PersonId, HashSet<PersonId>>>>,
}

impl PersonNetworkProjection {
    pub fn new() -> Self {
        Self {
            relationships: Arc::new(RwLock::new(HashMap::new())),
            adjacency_list: Arc::new(RwLock::new(HashMap::new())),
            reverse_adjacency: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Add or update a relationship
    async fn add_relationship(&self, relationship: PersonRelationship) {
        let mut relationships = self.relationships.write().await;
        let mut adjacency = self.adjacency_list.write().await;
        let mut reverse = self.reverse_adjacency.write().await;
        
        let key = (relationship.from_person, relationship.to_person);
        relationships.insert(key, relationship.clone());
        
        // Update adjacency lists
        adjacency.entry(relationship.from_person)
            .or_insert_with(HashSet::new)
            .insert(relationship.to_person);
        
        reverse.entry(relationship.to_person)
            .or_insert_with(HashSet::new)
            .insert(relationship.from_person);
    }
    
    /// Get direct connections for a person
    pub async fn get_connections(&self, person_id: &PersonId) -> Vec<PersonRelationship> {
        let relationships = self.relationships.read().await;
        let adjacency = self.adjacency_list.read().await;
        
        if let Some(connections) = adjacency.get(person_id) {
            connections.iter()
                .filter_map(|to_id| {
                    relationships.get(&(*person_id, *to_id)).cloned()
                })
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get people who have connections to this person
    pub async fn get_incoming_connections(&self, person_id: &PersonId) -> Vec<PersonRelationship> {
        let relationships = self.relationships.read().await;
        let reverse = self.reverse_adjacency.read().await;
        
        if let Some(connections) = reverse.get(person_id) {
            connections.iter()
                .filter_map(|from_id| {
                    relationships.get(&(*from_id, *person_id)).cloned()
                })
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get network statistics for a person
    pub async fn get_network_stats(&self, person_id: &PersonId) -> NetworkStats {
        let relationships = self.relationships.read().await;
        let adjacency = self.adjacency_list.read().await;
        let reverse = self.reverse_adjacency.read().await;
        
        let outgoing = adjacency.get(person_id).map(|s| s.len()).unwrap_or(0);
        let incoming = reverse.get(person_id).map(|s| s.len()).unwrap_or(0);
        let total_connections = outgoing + incoming;
        
        // Count connections by type
        let mut connections_by_type: HashMap<RelationshipType, usize> = HashMap::new();
        let mut total_strength = 0.0;
        let mut strength_count = 0;
        
        // Outgoing relationships
        if let Some(connections) = adjacency.get(person_id) {
            for to_id in connections {
                if let Some(rel) = relationships.get(&(*person_id, *to_id)) {
                    *connections_by_type.entry(rel.relationship_type.clone()).or_insert(0) += 1;
                    total_strength += rel.strength;
                    strength_count += 1;
                }
            }
        }
        
        // Incoming relationships
        if let Some(connections) = reverse.get(person_id) {
            for from_id in connections {
                if let Some(rel) = relationships.get(&(*from_id, *person_id)) {
                    *connections_by_type.entry(rel.relationship_type.clone()).or_insert(0) += 1;
                    total_strength += rel.strength;
                    strength_count += 1;
                }
            }
        }
        
        let average_relationship_strength = if strength_count > 0 {
            total_strength / strength_count as f32
        } else {
            0.0
        };
        
        // Calculate network reach (2nd degree connections)
        let network_reach = self.calculate_network_reach(person_id, &adjacency).await;
        
        // Calculate clustering coefficient
        let clustering_coefficient = self.calculate_clustering_coefficient(person_id, &adjacency).await;
        
        NetworkStats {
            person_id: *person_id,
            total_connections,
            connections_by_type,
            average_relationship_strength,
            network_reach,
            clustering_coefficient,
        }
    }
    
    /// Calculate 2nd degree connections
    async fn calculate_network_reach(
        &self,
        person_id: &PersonId,
        adjacency: &HashMap<PersonId, HashSet<PersonId>>
    ) -> usize {
        let mut second_degree = HashSet::new();
        
        if let Some(direct_connections) = adjacency.get(person_id) {
            for connection in direct_connections {
                if let Some(their_connections) = adjacency.get(connection) {
                    for second in their_connections {
                        if second != person_id && !direct_connections.contains(second) {
                            second_degree.insert(*second);
                        }
                    }
                }
            }
        }
        
        second_degree.len()
    }
    
    /// Calculate clustering coefficient (how connected are a person's connections)
    async fn calculate_clustering_coefficient(
        &self,
        person_id: &PersonId,
        adjacency: &HashMap<PersonId, HashSet<PersonId>>
    ) -> f32 {
        if let Some(connections) = adjacency.get(person_id) {
            let n = connections.len();
            if n < 2 {
                return 0.0;
            }
            
            let mut edges_between_neighbors = 0;
            let connections_vec: Vec<_> = connections.iter().collect();
            
            // Check how many of the person's connections are connected to each other
            for i in 0..connections_vec.len() {
                for j in (i + 1)..connections_vec.len() {
                    let person_a = connections_vec[i];
                    let person_b = connections_vec[j];
                    
                    if let Some(a_connections) = adjacency.get(person_a) {
                        if a_connections.contains(person_b) {
                            edges_between_neighbors += 1;
                        }
                    }
                    
                    if let Some(b_connections) = adjacency.get(person_b) {
                        if b_connections.contains(person_a) {
                            edges_between_neighbors += 1;
                        }
                    }
                }
            }
            
            let possible_edges = n * (n - 1);
            if possible_edges > 0 {
                edges_between_neighbors as f32 / possible_edges as f32
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
    
    /// Find shortest path between two people
    pub async fn find_shortest_path(
        &self,
        from: &PersonId,
        to: &PersonId
    ) -> Option<Vec<PersonId>> {
        let adjacency = self.adjacency_list.read().await;
        
        // BFS to find shortest path
        let mut queue = std::collections::VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent_map = HashMap::new();
        
        queue.push_back(*from);
        visited.insert(*from);
        
        while let Some(current) = queue.pop_front() {
            if current == *to {
                // Reconstruct path
                let mut path = vec![*to];
                let mut node = *to;
                
                while let Some(parent) = parent_map.get(&node) {
                    path.push(*parent);
                    node = *parent;
                }
                
                path.reverse();
                return Some(path);
            }
            
            if let Some(neighbors) = adjacency.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(*neighbor);
                        parent_map.insert(*neighbor, current);
                        queue.push_back(*neighbor);
                    }
                }
            }
        }
        
        None
    }
}

#[async_trait::async_trait]
impl PersonProjection for PersonNetworkProjection {
    async fn handle_event(&self, event: &PersonEvent) -> DomainResult<()> {
        match event {
            PersonEvent::ComponentDataUpdated(e) => {
                if let ComponentData::Social(social_data) = &e.data {
                    match social_data {
                        SocialData::Relationship(rel_data) => {
                            let relationship = PersonRelationship {
                                from_person: e.person_id,
                                to_person: rel_data.other_person_id,
                                relationship_type: match rel_data.relationship_type.as_str() {
                                    "family" => RelationshipType::Family,
                                    "friend" => RelationshipType::Friend,
                                    "colleague" => RelationshipType::Colleague,
                                    "manager" => RelationshipType::Manager,
                                    "report" => RelationshipType::Report,
                                    "mentor" => RelationshipType::Mentor,
                                    "mentee" => RelationshipType::Mentee,
                                    "business_partner" => RelationshipType::BusinessPartner,
                                    other => RelationshipType::Other(other.to_string()),
                                },
                                strength: 0.5, // Default strength
                                established_at: e.updated_at,
                                last_interaction: None,
                                interaction_count: 0,
                            };
                            
                            self.add_relationship(relationship).await;
                        }
                        _ => {} // Other social data types don't create relationships
                    }
                }
            }
            
            PersonEvent::PersonDeactivated(e) => {
                let mut relationships = self.relationships.write().await;
                let mut adjacency = self.adjacency_list.write().await;
                let mut reverse = self.reverse_adjacency.write().await;
                
                // Remove all relationships involving this person
                relationships.retain(|(from, to), _| {
                    from != &e.person_id && to != &e.person_id
                });
                
                // Remove from adjacency lists
                adjacency.remove(&e.person_id);
                for connections in adjacency.values_mut() {
                    connections.remove(&e.person_id);
                }
                
                reverse.remove(&e.person_id);
                for connections in reverse.values_mut() {
                    connections.remove(&e.person_id);
                }
            }
            
            PersonEvent::PersonMergedInto(e) => {
                // Transfer relationships to target person
                let mut relationships = self.relationships.write().await;
                let mut adjacency = self.adjacency_list.write().await;
                let mut reverse = self.reverse_adjacency.write().await;
                
                // Collect relationships to transfer
                let mut to_transfer = Vec::new();
                relationships.retain(|(from, to), rel| {
                    if from == &e.source_person_id {
                        let mut new_rel = rel.clone();
                        new_rel.from_person = e.merged_into_id;
                        to_transfer.push(((e.merged_into_id, *to), new_rel));
                        false
                    } else if to == &e.source_person_id {
                        let mut new_rel = rel.clone();
                        new_rel.to_person = e.merged_into_id;
                        to_transfer.push(((*from, e.merged_into_id), new_rel));
                        false
                    } else {
                        true
                    }
                });
                
                // Add transferred relationships
                for (key, rel) in to_transfer {
                    relationships.insert(key, rel);
                }
                
                // Update adjacency lists
                if let Some(connections) = adjacency.remove(&e.source_person_id) {
                    adjacency.entry(e.merged_into_id)
                        .or_insert_with(HashSet::new)
                        .extend(connections);
                }
                
                if let Some(connections) = reverse.remove(&e.source_person_id) {
                    reverse.entry(e.merged_into_id)
                        .or_insert_with(HashSet::new)
                        .extend(connections);
                }
            }
            
            _ => {} // Other events don't affect network
        }
        
        Ok(())
    }
    
    fn projection_name(&self) -> &str {
        "PersonNetworkProjection"
    }
    
    async fn clear(&self) -> DomainResult<()> {
        let mut relationships = self.relationships.write().await;
        let mut adjacency = self.adjacency_list.write().await;
        let mut reverse = self.reverse_adjacency.write().await;
        
        relationships.clear();
        adjacency.clear();
        reverse.clear();
        
        Ok(())
    }
} 