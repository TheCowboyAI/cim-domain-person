//! Network analysis service for professional relationships

use crate::value_objects::{ProfessionalNetworkRelation, ProfessionalRelationType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

/// Network analysis service for analyzing professional relationships
pub struct NetworkAnalysisService {
    /// Graph of professional relationships
    relationships: HashMap<Uuid, Vec<ProfessionalNetworkRelation>>,
}

/// Network metrics for a person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// Person ID
    pub person_id: Uuid,
    /// Number of direct connections
    pub direct_connections: usize,
    /// Number of second-degree connections
    pub second_degree_connections: usize,
    /// Clustering coefficient (how connected their connections are)
    pub clustering_coefficient: f32,
    /// Betweenness centrality (how often they're on shortest paths)
    pub betweenness_centrality: f32,
    /// Influence score (composite metric)
    pub influence_score: f32,
    /// Most connected to (top 5)
    pub strongest_connections: Vec<(Uuid, f32)>,
    /// Connection diversity by type
    pub connection_diversity: HashMap<String, usize>,
}

/// Path between two people in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPath {
    /// Start person
    pub from: Uuid,
    /// End person
    pub to: Uuid,
    /// Path of person IDs
    pub path: Vec<Uuid>,
    /// Total strength (product of edge strengths)
    pub strength: f32,
}

/// Community detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCommunity {
    /// Community ID
    pub id: Uuid,
    /// Members of the community
    pub members: HashSet<Uuid>,
    /// Community cohesion score
    pub cohesion: f32,
    /// Primary relationship types in community
    pub primary_types: Vec<String>,
}

impl NetworkAnalysisService {
    pub fn new() -> Self {
        Self {
            relationships: HashMap::new(),
        }
    }
    
    /// Add a relationship to the network
    pub fn add_relationship(&mut self, from: Uuid, relation: ProfessionalNetworkRelation) {
        self.relationships
            .entry(from)
            .or_insert_with(Vec::new)
            .push(relation.clone());
        
        // Add reverse relationship for undirected graph
        let reverse = ProfessionalNetworkRelation {
            other_person_id: from,
            ..relation
        };
        
        self.relationships
            .entry(relation.other_person_id)
            .or_insert_with(Vec::new)
            .push(reverse);
    }
    
    /// Calculate network metrics for a person
    pub fn calculate_metrics(&self, person_id: Uuid) -> NetworkMetrics {
        let direct_connections = self.get_direct_connections(person_id);
        let second_degree = self.get_second_degree_connections(person_id);
        let clustering = self.calculate_clustering_coefficient(person_id);
        let betweenness = self.calculate_betweenness_centrality(person_id);
        let strongest = self.get_strongest_connections(person_id, 5);
        let diversity = self.calculate_connection_diversity(person_id);
        
        let influence_score = self.calculate_influence_score(
            direct_connections.len(),
            second_degree.len(),
            clustering,
            betweenness,
        );
        
        NetworkMetrics {
            person_id,
            direct_connections: direct_connections.len(),
            second_degree_connections: second_degree.len(),
            clustering_coefficient: clustering,
            betweenness_centrality: betweenness,
            influence_score,
            strongest_connections: strongest,
            connection_diversity: diversity,
        }
    }
    
    /// Get direct connections
    fn get_direct_connections(&self, person_id: Uuid) -> HashSet<Uuid> {
        self.relationships
            .get(&person_id)
            .map(|rels| rels.iter().map(|r| r.other_person_id).collect())
            .unwrap_or_default()
    }
    
    /// Get second-degree connections
    fn get_second_degree_connections(&self, person_id: Uuid) -> HashSet<Uuid> {
        let mut second_degree = HashSet::new();
        let direct = self.get_direct_connections(person_id);
        
        for connection in &direct {
            if let Some(their_connections) = self.relationships.get(connection) {
                for rel in their_connections {
                    if rel.other_person_id != person_id && !direct.contains(&rel.other_person_id) {
                        second_degree.insert(rel.other_person_id);
                    }
                }
            }
        }
        
        second_degree
    }
    
    /// Calculate clustering coefficient
    fn calculate_clustering_coefficient(&self, person_id: Uuid) -> f32 {
        let connections = self.get_direct_connections(person_id);
        if connections.len() < 2 {
            return 0.0;
        }
        
        let mut connected_pairs = 0;
        let connections_vec: Vec<_> = connections.iter().cloned().collect();
        
        for i in 0..connections_vec.len() {
            for j in (i + 1)..connections_vec.len() {
                if self.are_connected(connections_vec[i], connections_vec[j]) {
                    connected_pairs += 1;
                }
            }
        }
        
        let possible_pairs = connections.len() * (connections.len() - 1) / 2;
        connected_pairs as f32 / possible_pairs as f32
    }
    
    /// Check if two people are connected
    fn are_connected(&self, person1: Uuid, person2: Uuid) -> bool {
        self.relationships
            .get(&person1)
            .map(|rels| rels.iter().any(|r| r.other_person_id == person2))
            .unwrap_or(false)
    }
    
    /// Calculate betweenness centrality (simplified)
    fn calculate_betweenness_centrality(&self, person_id: Uuid) -> f32 {
        // Simplified: count how many shortest paths go through this person
        let mut paths_through = 0;
        let all_people: HashSet<_> = self.relationships.keys().cloned().collect();
        
        for &source in &all_people {
            for &target in &all_people {
                if source != target && source != person_id && target != person_id {
                    if let Some(path) = self.find_shortest_path(source, target) {
                        if path.path.contains(&person_id) {
                            paths_through += 1;
                        }
                    }
                }
            }
        }
        
        let total_pairs = all_people.len() * (all_people.len() - 1);
        paths_through as f32 / total_pairs as f32
    }
    
    /// Find shortest path between two people
    pub fn find_shortest_path(&self, from: Uuid, to: Uuid) -> Option<NetworkPath> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent = HashMap::new();
        
        queue.push_back(from);
        visited.insert(from);
        
        while let Some(current) = queue.pop_front() {
            if current == to {
                // Reconstruct path
                let mut path = vec![to];
                let mut node = to;
                
                while node != from {
                    if let Some(&p) = parent.get(&node) {
                        path.push(p);
                        node = p;
                    } else {
                        break;
                    }
                }
                
                path.reverse();
                
                // Calculate path strength
                let strength = self.calculate_path_strength(&path);
                
                return Some(NetworkPath {
                    from,
                    to,
                    path,
                    strength,
                });
            }
            
            if let Some(relations) = self.relationships.get(&current) {
                for rel in relations {
                    if !visited.contains(&rel.other_person_id) {
                        visited.insert(rel.other_person_id);
                        parent.insert(rel.other_person_id, current);
                        queue.push_back(rel.other_person_id);
                    }
                }
            }
        }
        
        None
    }
    
    /// Calculate strength of a path
    fn calculate_path_strength(&self, path: &[Uuid]) -> f32 {
        if path.len() < 2 {
            return 0.0;
        }
        
        let mut strength = 1.0;
        
        for i in 0..path.len() - 1 {
            if let Some(relations) = self.relationships.get(&path[i]) {
                if let Some(rel) = relations.iter().find(|r| r.other_person_id == path[i + 1]) {
                    strength *= rel.strength;
                }
            }
        }
        
        strength
    }
    
    /// Get strongest connections
    fn get_strongest_connections(&self, person_id: Uuid, count: usize) -> Vec<(Uuid, f32)> {
        let mut connections: Vec<_> = self.relationships
            .get(&person_id)
            .map(|rels| {
                rels.iter()
                    .map(|r| (r.other_person_id, r.influence_score()))
                    .collect()
            })
            .unwrap_or_default();
        
        connections.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        connections.truncate(count);
        connections
    }
    
    /// Calculate connection diversity
    fn calculate_connection_diversity(&self, person_id: Uuid) -> HashMap<String, usize> {
        let mut diversity = HashMap::new();
        
        if let Some(relations) = self.relationships.get(&person_id) {
            for rel in relations {
                let type_name = match &rel.relation_type {
                    ProfessionalRelationType::Colleague { .. } => "Colleague",
                    ProfessionalRelationType::Manager => "Manager",
                    ProfessionalRelationType::Subordinate => "Subordinate",
                    ProfessionalRelationType::Mentor => "Mentor",
                    ProfessionalRelationType::Mentee => "Mentee",
                    ProfessionalRelationType::BusinessPartner => "BusinessPartner",
                    ProfessionalRelationType::Client => "Client",
                    ProfessionalRelationType::Vendor => "Vendor",
                    ProfessionalRelationType::ProfessionalContact => "ProfessionalContact",
                    ProfessionalRelationType::Alumni { .. } => "Alumni",
                    ProfessionalRelationType::ConferenceContact { .. } => "ConferenceContact",
                    ProfessionalRelationType::Other(_) => "Other",
                };
                
                *diversity.entry(type_name.to_string()).or_insert(0) += 1;
            }
        }
        
        diversity
    }
    
    /// Calculate influence score
    fn calculate_influence_score(
        &self,
        direct_connections: usize,
        second_degree: usize,
        clustering: f32,
        betweenness: f32,
    ) -> f32 {
        let connection_score = (direct_connections as f32).ln() / 10.0;
        let reach_score = (second_degree as f32).ln() / 20.0;
        let quality_score = clustering * 0.3;
        let bridge_score = betweenness * 0.4;
        
        (connection_score + reach_score + quality_score + bridge_score).min(1.0)
    }
    
    /// Detect communities using simple algorithm
    pub fn detect_communities(&self, min_size: usize) -> Vec<NetworkCommunity> {
        // Simple community detection based on connected components
        let mut communities = Vec::new();
        let mut visited = HashSet::new();
        
        for &person_id in self.relationships.keys() {
            if !visited.contains(&person_id) {
                let community = self.explore_community(person_id, &mut visited);
                
                if community.len() >= min_size {
                    let cohesion = self.calculate_community_cohesion(&community);
                    let types = self.get_community_types(&community);
                    
                    communities.push(NetworkCommunity {
                        id: Uuid::new_v4(),
                        members: community,
                        cohesion,
                        primary_types: types,
                    });
                }
            }
        }
        
        communities
    }
    
    /// Explore a community starting from a person
    fn explore_community(&self, start: Uuid, visited: &mut HashSet<Uuid>) -> HashSet<Uuid> {
        let mut community = HashSet::new();
        let mut queue = VecDeque::new();
        
        queue.push_back(start);
        visited.insert(start);
        community.insert(start);
        
        while let Some(current) = queue.pop_front() {
            if let Some(relations) = self.relationships.get(&current) {
                for rel in relations {
                    if !visited.contains(&rel.other_person_id) && rel.strength > 0.5 {
                        visited.insert(rel.other_person_id);
                        community.insert(rel.other_person_id);
                        queue.push_back(rel.other_person_id);
                    }
                }
            }
        }
        
        community
    }
    
    /// Calculate community cohesion
    fn calculate_community_cohesion(&self, community: &HashSet<Uuid>) -> f32 {
        if community.len() < 2 {
            return 0.0;
        }
        
        let mut internal_edges = 0;
        let mut total_strength = 0.0;
        
        for &member in community {
            if let Some(relations) = self.relationships.get(&member) {
                for rel in relations {
                    if community.contains(&rel.other_person_id) {
                        internal_edges += 1;
                        total_strength += rel.strength;
                    }
                }
            }
        }
        
        let possible_edges = community.len() * (community.len() - 1);
        let density = internal_edges as f32 / possible_edges as f32;
        let avg_strength = total_strength / internal_edges as f32;
        
        density * avg_strength
    }
    
    /// Get primary relationship types in a community
    fn get_community_types(&self, community: &HashSet<Uuid>) -> Vec<String> {
        let mut type_counts = HashMap::new();
        
        for &member in community {
            if let Some(diversity) = self.calculate_connection_diversity(member).iter().max_by_key(|&(_, v)| v) {
                *type_counts.entry(diversity.0.clone()).or_insert(0) += 1;
            }
        }
        
        let mut types: Vec<_> = type_counts.into_iter().collect();
        types.sort_by_key(|&(_, count)| std::cmp::Reverse(count));
        types.truncate(3);
        
        types.into_iter().map(|(t, _)| t).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_network_metrics() {
        let mut service = NetworkAnalysisService::new();
        
        let person1 = Uuid::new_v4();
        let person2 = Uuid::new_v4();
        let person3 = Uuid::new_v4();
        
        // Create a triangle network
        service.add_relationship(person1, ProfessionalNetworkRelation {
            other_person_id: person2,
            relation_type: ProfessionalRelationType::Colleague { same_team: true, same_department: true },
            strength: 0.8,
            established_date: chrono::Utc::now(),
            last_interaction: Some(chrono::Utc::now()),
            interaction_count: 10,
            mutual_connections: 1,
        });
        
        service.add_relationship(person2, ProfessionalNetworkRelation {
            other_person_id: person3,
            relation_type: ProfessionalRelationType::Colleague { same_team: false, same_department: true },
            strength: 0.6,
            established_date: chrono::Utc::now(),
            last_interaction: Some(chrono::Utc::now()),
            interaction_count: 5,
            mutual_connections: 1,
        });
        
        service.add_relationship(person1, ProfessionalNetworkRelation {
            other_person_id: person3,
            relation_type: ProfessionalRelationType::Mentor,
            strength: 0.9,
            established_date: chrono::Utc::now(),
            last_interaction: Some(chrono::Utc::now()),
            interaction_count: 20,
            mutual_connections: 1,
        });
        
        let metrics = service.calculate_metrics(person1);
        
        assert_eq!(metrics.direct_connections, 2);
        assert_eq!(metrics.clustering_coefficient, 1.0); // Perfect triangle
        assert!(metrics.influence_score > 0.0);
    }
    
    #[test]
    fn test_shortest_path() {
        let mut service = NetworkAnalysisService::new();
        
        let person1 = Uuid::new_v4();
        let person2 = Uuid::new_v4();
        let person3 = Uuid::new_v4();
        let person4 = Uuid::new_v4();
        
        // Create a path: 1 -> 2 -> 3 -> 4
        service.add_relationship(person1, ProfessionalNetworkRelation {
            other_person_id: person2,
            relation_type: ProfessionalRelationType::Colleague { same_team: true, same_department: true },
            strength: 0.8,
            established_date: chrono::Utc::now(),
            last_interaction: Some(chrono::Utc::now()),
            interaction_count: 10,
            mutual_connections: 0,
        });
        
        service.add_relationship(person2, ProfessionalNetworkRelation {
            other_person_id: person3,
            relation_type: ProfessionalRelationType::Manager,
            strength: 0.7,
            established_date: chrono::Utc::now(),
            last_interaction: Some(chrono::Utc::now()),
            interaction_count: 15,
            mutual_connections: 0,
        });
        
        service.add_relationship(person3, ProfessionalNetworkRelation {
            other_person_id: person4,
            relation_type: ProfessionalRelationType::BusinessPartner,
            strength: 0.6,
            established_date: chrono::Utc::now(),
            last_interaction: Some(chrono::Utc::now()),
            interaction_count: 5,
            mutual_connections: 0,
        });
        
        let path = service.find_shortest_path(person1, person4).unwrap();
        
        assert_eq!(path.path.len(), 4);
        assert_eq!(path.path[0], person1);
        assert_eq!(path.path[3], person4);
        assert!(path.strength > 0.0);
    }
} 