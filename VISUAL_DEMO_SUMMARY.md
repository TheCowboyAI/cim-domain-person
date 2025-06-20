# Person Domain Visual Relationship Demos

## Overview

We've created three powerful visual demonstrations of the Person domain's relationship capabilities:

## 1. Relationship Network Demo (`relationship_network_demo.rs`)

**Features**:
- Creates a complete organizational hierarchy with CEO, executives, employees
- Includes customers, influencers, and business partners
- Shows different relationship types (manages, collaborates, refers, partners)
- Generates multiple visualization formats

**Output Formats**:

### ASCII Art Visualization
```
                    ┌─────────────────────┐
                    │ CEO: Victoria       │
                    │ 37K followers       │
                    └──────┬──────────────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
    ┌────▼────┐      ┌────▼────┐      ┌────▼────┐
    │ CTO     │      │ CMO     │      │ CFO     │
    │ Marcus  │      │ David   │      │ Rachel  │
    └────┬────┘      └────┬────┘      └─────────┘
```

### Mermaid Diagram
- Color-coded by role (executives in red, employees in teal, customers in blue)
- Shows follower counts for social media influencers
- Different arrow styles for relationship types
- Can be pasted into any Mermaid-compatible viewer

### DOT/Graphviz Format
- Professional graph layout
- Suitable for high-quality PDF/SVG export
- Includes all metadata in node labels

## 2. Interactive HTML Export (`interactive_graph_export.rs`)

**Features**:
- Generates a standalone HTML file with embedded JavaScript
- Uses vis.js library for interactive visualization
- No external dependencies needed

**Interactive Capabilities**:
- 🖱️ **Drag nodes** - Rearrange the layout by dragging
- 🔍 **Zoom** - Mouse wheel to zoom in/out
- 👆 **Click to highlight** - Click nodes to see connections
- 📍 **Hover details** - See role and name on hover
- 🎨 **Color coding** - Different colors for roles

**Generated Files**:
- `person_relationship_graph.html` - Interactive visualization
- `person_relationship_data.json` - Raw data export

## 3. Network Statistics and Insights

Both demos provide valuable analytics:

### Network Statistics
```
📊 Network Statistics:
  Total People: 9
  Total Relationships: 10
  
  People by Role:
    - CEO: 1
    - CTO: 1
    - Developers: 2
    - Customers: 2
    ...
```

### Relationship Insights
```
💡 Relationship Insights:
  🌟 Most Connected: Victoria Sterling (CEO) with 4 connections
  📱 Social Media Influencers:
    - Sarah Tech: 525K followers, 8.2% engagement
    - Victoria Sterling: 37K followers, 4.5% engagement
  💰 High-Value Relationships:
    - TechCorp Inc: $2000K predicted LTV
```

## Use Cases

### 1. Organizational Charts
- Visualize reporting structures
- Identify key connectors and influencers
- Plan reorganizations

### 2. Customer Relationship Mapping
- See customer-employee relationships
- Track referral networks
- Identify high-value connections

### 3. Social Network Analysis
- Find influencers by follower count
- Track engagement rates
- Map social connections

### 4. Business Intelligence
- Visualize partner networks
- Track business relationships
- Analyze relationship patterns

## Running the Demos

```bash
# ASCII/Mermaid/DOT visualization
cargo run --example relationship_network_demo

# Interactive HTML export
cargo run --example interactive_graph_export

# Then open in browser
open person_relationship_graph.html  # macOS
xdg-open person_relationship_graph.html  # Linux
```

## Integration with CIM Graph Domain

These demos showcase how the Person domain can integrate with the CIM Graph domain for full visualization capabilities:

1. **Person entities** → Graph nodes
2. **Relationships** → Graph edges
3. **Roles/segments** → Node types/colors
4. **Social metrics** → Node sizes/labels

The Person domain provides the rich data model, while the Graph domain would provide the full visualization engine in a production CIM system.

## Future Enhancements

1. **Real-time Updates** - WebSocket connection for live graph updates
2. **3D Visualization** - Using Bevy for full 3D relationship graphs
3. **Graph Analytics** - Centrality, clustering, shortest paths
4. **Export Options** - PDF, SVG, PNG export capabilities
5. **Filtering** - Show/hide by role, relationship type, or metrics

## Summary

The Person domain now has comprehensive visual relationship capabilities that can:
- Generate multiple visualization formats
- Create interactive web-based graphs
- Provide network analytics and insights
- Export data for other visualization tools

This makes it easy to understand complex organizational and social networks at a glance! 