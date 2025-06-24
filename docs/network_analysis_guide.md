# Network Analysis Guide

## Overview

The Person domain provides powerful capabilities for analyzing networks and relationships between people, organizations, locations, and policies. This guide covers patterns and techniques for leveraging these capabilities.

## Relationship Types

### Person-to-Person Relationships

```rust
pub enum PersonRelationshipType {
    // Professional
    Manager,
    DirectReport,
    Colleague,
    Mentor,
    Mentee,
    
    // Business
    Partner,
    Client,
    Vendor,
    Advisor,
    
    // Social
    Friend,
    Acquaintance,
    Influencer,
    Follower,
    
    // Family (if applicable)
    Spouse,
    Parent,
    Child,
    Sibling,
}
```

### Person-Organization Relationships

```rust
pub enum OrganizationRelationType {
    Employee,
    Contractor,
    Consultant,
    BoardMember,
    Advisor,
    Partner,
    Customer,
    Vendor,
    Alumni,
}
```

### Person-Location Relationships

```rust
pub enum LocationRelationType {
    Residence,
    WorkLocation,
    MailingAddress,
    BillingAddress,
    FrequentVisitor,
    PastResident,
}
```

## Network Analysis Patterns

### 1. Organizational Hierarchy Analysis

```rust
/// Find all reports for a manager (direct and indirect)
pub async fn find_organizational_tree(
    person_id: PersonId,
    max_depth: Option<u32>,
) -> Result<OrganizationalTree> {
    let mut tree = OrganizationalTree::new(person_id);
    let mut current_level = vec![person_id];
    let mut depth = 0;
    
    while !current_level.is_empty() && 
          depth < max_depth.unwrap_or(u32::MAX) {
        let mut next_level = Vec::new();
        
        for manager_id in current_level {
            let reports = query_direct_reports(manager_id).await?;
            for report in reports {
                tree.add_relationship(manager_id, report.person_id);
                next_level.push(report.person_id);
            }
        }
        
        current_level = next_level;
        depth += 1;
    }
    
    Ok(tree)
}
```

### 2. Influence Network Mapping

```rust
/// Calculate influence scores based on network centrality
pub struct InfluenceAnalyzer {
    graph: Graph<PersonId, RelationshipStrength>,
}

impl InfluenceAnalyzer {
    pub fn calculate_influence_score(
        &self,
        person_id: PersonId,
    ) -> InfluenceScore {
        // Factors:
        // 1. Number of connections
        // 2. Quality of connections (to other influencers)
        // 3. Betweenness centrality
        // 4. Communication frequency
        
        let degree_centrality = self.calculate_degree_centrality(person_id);
        let betweenness = self.calculate_betweenness(person_id);
        let eigenvector = self.calculate_eigenvector_centrality(person_id);
        
        InfluenceScore {
            overall: (degree_centrality * 0.3 + 
                     betweenness * 0.4 + 
                     eigenvector * 0.3),
            degree_centrality,
            betweenness_centrality: betweenness,
            eigenvector_centrality: eigenvector,
        }
    }
}
```

### 3. Collaboration Networks

```rust
/// Find potential collaboration opportunities
pub async fn find_collaboration_opportunities(
    person_id: PersonId,
    skill_requirements: Vec<String>,
) -> Result<Vec<CollaborationOpportunity>> {
    // Find people with complementary skills
    let skill_matches = find_people_with_skills(&skill_requirements).await?;
    
    // Calculate collaboration scores
    let mut opportunities = Vec::new();
    
    for candidate in skill_matches {
        // Check existing relationships
        let existing_path = find_shortest_path(person_id, candidate.id).await?;
        
        // Calculate compatibility
        let compatibility = calculate_compatibility(
            person_id,
            candidate.id,
            &skill_requirements,
        ).await?;
        
        opportunities.push(CollaborationOpportunity {
            person: candidate,
            connection_path: existing_path,
            compatibility_score: compatibility,
            missing_skills: identify_skill_gaps(&skill_requirements, &candidate),
        });
    }
    
    // Sort by opportunity score
    opportunities.sort_by(|a, b| 
        b.opportunity_score().partial_cmp(&a.opportunity_score()).unwrap()
    );
    
    Ok(opportunities)
}
```

### 4. Geographic Distribution Analysis

```rust
/// Analyze geographic distribution of network
pub async fn analyze_geographic_distribution(
    root_person: PersonId,
    relationship_types: Vec<PersonRelationshipType>,
) -> Result<GeographicAnalysis> {
    let network = load_person_network(root_person, &relationship_types).await?;
    
    let mut location_clusters = HashMap::new();
    
    for person_id in network.all_people() {
        if let Some(location) = get_primary_location(person_id).await? {
            location_clusters
                .entry(location.region())
                .or_insert_with(Vec::new)
                .push(person_id);
        }
    }
    
    Ok(GeographicAnalysis {
        clusters: location_clusters,
        geographic_reach: calculate_geographic_spread(&location_clusters),
        remote_collaboration_pairs: find_remote_pairs(&network, &location_clusters),
    })
}
```

## Query Patterns

### Finding Network Paths

```rust
/// Find shortest path between two people
pub async fn find_connection_path(
    from: PersonId,
    to: PersonId,
    max_hops: u32,
) -> Result<Option<ConnectionPath>> {
    use petgraph::algo::dijkstra;
    
    let graph = load_relationship_graph().await?;
    let paths = dijkstra(
        &graph,
        from,
        Some(to),
        |_| 1, // Unit weight for each hop
    );
    
    if let Some(&distance) = paths.get(&to) {
        if distance <= max_hops as i32 {
            let path = reconstruct_path(&graph, from, to);
            return Ok(Some(ConnectionPath {
                nodes: path,
                distance,
                relationship_types: extract_relationship_types(&path),
            }));
        }
    }
    
    Ok(None)
}
```

### Network Density Analysis

```rust
/// Calculate network density for a group
pub fn calculate_network_density(
    people: &[PersonId],
) -> NetworkDensity {
    let total_possible = people.len() * (people.len() - 1) / 2;
    let actual_connections = count_connections_between(people);
    
    NetworkDensity {
        density: actual_connections as f64 / total_possible as f64,
        total_people: people.len(),
        total_connections: actual_connections,
        average_connections: actual_connections as f64 / people.len() as f64,
    }
}
```

### Community Detection

```rust
/// Detect communities within a network
pub async fn detect_communities(
    seed_people: Vec<PersonId>,
    min_community_size: usize,
) -> Result<Vec<Community>> {
    let graph = build_extended_network(&seed_people).await?;
    
    // Use Louvain algorithm for community detection
    let communities = louvain_communities(&graph);
    
    communities
        .into_iter()
        .filter(|c| c.members.len() >= min_community_size)
        .map(|c| Community {
            id: CommunityId::new(),
            members: c.members,
            density: calculate_network_density(&c.members),
            central_figures: identify_central_figures(&c),
            common_attributes: find_common_attributes(&c.members).await?,
        })
        .collect()
}
```

## Performance Optimization

### 1. Caching Strategies

```rust
pub struct NetworkCache {
    // Cache frequently accessed paths
    path_cache: LruCache<(PersonId, PersonId), ConnectionPath>,
    
    // Cache influence scores with TTL
    influence_cache: TtlCache<PersonId, InfluenceScore>,
    
    // Cache network snapshots
    network_snapshots: HashMap<NetworkQuery, CachedNetwork>,
}
```

### 2. Batch Processing

```rust
/// Process network queries in batches
pub async fn batch_network_analysis(
    queries: Vec<NetworkQuery>,
) -> Result<Vec<NetworkResult>> {
    // Group similar queries
    let grouped = group_queries_by_type(queries);
    
    // Process each group in parallel
    let futures: Vec<_> = grouped
        .into_iter()
        .map(|(query_type, queries)| {
            tokio::spawn(async move {
                process_query_batch(query_type, queries).await
            })
        })
        .collect();
    
    // Collect results
    let results = futures::future::join_all(futures).await;
    
    Ok(flatten_results(results))
}
```

### 3. Graph Indexing

```rust
pub struct NetworkIndex {
    // Spatial index for geographic queries
    location_index: RTree<PersonLocation>,
    
    // Skill index for expertise queries
    skill_index: HashMap<SkillId, HashSet<PersonId>>,
    
    // Relationship index for fast traversal
    relationship_index: AdjacencyList<PersonId>,
    
    // Influence index for ranking
    influence_rankings: BTreeMap<InfluenceScore, PersonId>,
}
```

## Visualization Integration

### Network Visualization Data

```rust
/// Prepare network data for visualization
pub struct NetworkVisualization {
    pub nodes: Vec<NodeData>,
    pub edges: Vec<EdgeData>,
    pub layout: LayoutAlgorithm,
    pub filters: NetworkFilters,
}

impl NetworkVisualization {
    pub fn prepare_for_bevy(&self) -> BevyNetworkData {
        BevyNetworkData {
            nodes: self.nodes.iter().map(|n| BevyNode {
                entity_id: n.person_id,
                position: self.calculate_position(n),
                color: self.calculate_color(n),
                size: n.influence_score * 10.0,
                label: n.display_name.clone(),
            }).collect(),
            
            edges: self.edges.iter().map(|e| BevyEdge {
                source: e.from,
                target: e.to,
                strength: e.relationship_strength,
                edge_type: e.relationship_type,
            }).collect(),
        }
    }
}
```

## Best Practices

1. **Privacy Considerations**
   - Always check privacy preferences before exposing relationships
   - Implement relationship visibility levels
   - Audit all network queries

2. **Performance**
   - Use pagination for large networks
   - Implement caching for expensive calculations
   - Consider graph databases for complex queries

3. **Data Quality**
   - Validate relationships bidirectionally
   - Handle relationship conflicts
   - Maintain relationship history

4. **Scalability**
   - Partition large networks by region or organization
   - Use streaming algorithms for real-time analysis
   - Implement incremental updates 