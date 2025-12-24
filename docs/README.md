# Documentation

> **Languages**: [English](#english-documentation) | [Français](#documentation-française)

Welcome to the 3D Print Quote Service documentation. This page provides an overview of all available documentation.

---

## English Documentation

### 📚 Main Topics

| Topic | Description | Link |
|-------|-------------|------|
| **Architecture** | System design, data flows, and technical decisions | [architecture.md](en/architecture.md) |
| **MCP Integration** | Model Context Protocol for AI/automation | [mcp.md](en/mcp.md) |

### 📖 Technical References

| Document | Description |
|----------|-------------|
| [API Specification](api.yaml) | OpenAPI/Swagger specification |
| [Security Audit](SECURITY_AUDIT.md) | Security considerations and threat model |
| [ADR-005](ADR-005-postgresql-only.md) | Architecture Decision Record: PostgreSQL only |

### 🔍 Reviews & Analysis

| Document | Date | Description |
|----------|------|-------------|
| [Code Review](reviews/code_review_2025-12-23.md) | 2025-12-23 | Comprehensive code review findings |

---

## Documentation Française

### 📚 Sujets Principaux

| Sujet | Description | Lien |
|-------|-------------|------|
| **Architecture** | Conception système, flux de données, décisions techniques | [architecture.md](fr/architecture.md) |
| **Intégration MCP** | Model Context Protocol pour IA/automation | [mcp.md](fr/mcp.md) |

### 📖 Références Techniques

| Document | Description |
|----------|-------------|
| [Spécification API](api.yaml) | Spécification OpenAPI/Swagger |
| [Audit Sécurité](SECURITY_AUDIT.md) | Considérations de sécurité et modèle de menace |
| [ADR-005](ADR-005-postgresql-only.md) | Architecture Decision Record: PostgreSQL uniquement |

### 🔍 Revues & Analyses

| Document | Date | Description |
|----------|------|-------------|
| [Code Review](reviews/code_review_2025-12-23.md) | 2025-12-23 | Résultats de la revue de code complète |

---

## Quick Start

### For Users
1. Start with the main [README](../README.md) for installation instructions
2. Check the [Architecture documentation](en/architecture.md) to understand the system
3. Use the web interface at `http://localhost:3000` after installation

### For Developers
1. Read the [Architecture documentation](en/architecture.md) for system overview
2. Check [ADR-005](ADR-005-postgresql-only.md) for architectural decisions
3. Review the [API specification](api.yaml) for endpoint details
4. Read the [Code Review](reviews/code_review_2025-12-23.md) for quality insights

### For AI/Automation
1. Read the [MCP documentation](en/mcp.md) for programmatic access
2. Check the [API specification](api.yaml) for endpoint contracts
3. Review authentication requirements in the [Security Audit](SECURITY_AUDIT.md)

---

## Documentation Structure

```
docs/
├── README.md                        # This file - Documentation hub
│
├── en/                              # English documentation
│   ├── architecture.md              # System architecture
│   └── mcp.md                       # MCP integration guide
│
├── fr/                              # French documentation
│   ├── architecture.md              # Architecture système
│   └── mcp.md                       # Guide intégration MCP
│
├── architecture/
│   └── README.md                    # Architecture hub (redirects to en/fr)
│
├── MCP.md                           # MCP hub (redirects to en/fr)
│
├── reviews/
│   └── code_review_2025-12-23.md   # Code review reports
│
├── api.yaml                         # OpenAPI specification
├── SECURITY_AUDIT.md               # Security documentation
└── ADR-005-postgresql-only.md      # Architecture decision record
```

---

## Contributing to Documentation

When adding new documentation:

1. **Add both English and French versions**
   - Create `docs/en/your-topic.md`
   - Create `docs/fr/your-topic.md`

2. **Update this index**
   - Add entry to the appropriate table
   - Use consistent formatting

3. **Create a hub file if needed**
   - Create `docs/your-topic.md` with links to en/fr versions
   - Follow the pattern used by `MCP.md` and `architecture/README.md`

4. **Follow the style guide**
   - Use clear headings
   - Include code examples
   - Add diagrams when helpful
   - Keep language simple and technical

---

## Need Help?

- **Installation issues**: Check the main [README](../README.md)
- **API questions**: See [api.yaml](api.yaml) or [MCP documentation](en/mcp.md)
- **Architecture questions**: See [architecture documentation](en/architecture.md)
- **Security concerns**: Review [SECURITY_AUDIT.md](SECURITY_AUDIT.md)
- **Report issues**: https://github.com/ziggornif/3d-assistant/issues
