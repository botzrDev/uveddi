# Release Notes

This directory contains comprehensive release documentation for Uveddi, providing users with detailed information about versions, changes, and known issues.

## Documents Overview

### [📋 Changelog](./changelog.md)
Complete version history with detailed changes for each release. Includes:
- Feature additions and improvements
- Bug fixes and security updates
- Breaking changes and migration notes
- Performance enhancements
- Dependency updates

### [⚠️ Known Issues](./known-issues.md)
Current limitations and known problems across all versions. Organized by:
- Critical issues affecting core functionality
- Component-specific limitations
- Workarounds and mitigation strategies
- Timeline for resolution

### [🎯 Version Status](./v1.0-alpha-status.md)
Detailed status report for the current v1.0-alpha release including:
- Feature maturity matrix
- Production readiness assessment
- Performance characteristics
- Security status
- Roadmap to v1.0 GA

## Version Navigation

### Current Releases
- **v1.0-alpha** (Current) - Production-ready core with alpha web features
- **v0.9.0-alpha** (Legacy) - Initial alpha release

### Upcoming Releases
- **v1.0-beta** - Targeted for Q1 2026
- **v1.0 GA** - Targeted for Q3 2026

## Quick Reference

### Production Readiness
| Component | v0.9.0-alpha | v1.0-alpha | v1.0-beta | v1.0 GA |
|-----------|--------------|------------|-----------|---------|
| CLI Engine | ✅ Ready | ✅ Ready | ✅ Ready | ✅ Ready |
| API Server | ⚠️ Alpha | ✅ Ready | ✅ Ready | ✅ Ready |
| Web Dashboard | ❌ Limited | ⚠️ Alpha | ✅ Ready | ✅ Ready |
| Plugin System | ❌ Development | ⚠️ Alpha | ✅ Ready | ✅ Ready |
| TypeScript Support | ❌ Limited | ⚠️ Alpha | ✅ Ready | ✅ Ready |

### Language Support
- **✅ Production Ready**: Rust, Python, JavaScript
- **⚠️ Alpha Support**: TypeScript (basic parsing)
- **📋 Planned**: Java, C#, Go (v1.1+)

### Key Features Status
- **Static Analysis**: ✅ Production ready
- **Anti-Pattern Detection**: ✅ Production ready  
- **Report Generation**: ✅ Production ready (JSON, HTML, Markdown)
- **Database Integration**: ✅ Production ready (PostgreSQL)
- **Monitoring**: ✅ Production ready (Prometheus metrics)
- **Security**: ⚠️ Basic JWT auth, enterprise features planned

## Getting Started

### For New Users
1. Check the [changelog](./changelog.md) for the latest features
2. Review [known issues](./known-issues.md) for current limitations
3. See the [v1.0-alpha status](./v1.0-alpha-status.md) for detailed capability assessment

### For Existing Users
- **Upgrading from v0.9.0**: Full backward compatibility, automatic migrations
- **Migration Path**: See individual version entries in changelog for specific upgrade notes

### For Production Deployment
- **Recommended Version**: v1.0-alpha for CLI/API usage
- **Web Dashboard**: Alpha quality - suitable for basic report viewing
- **Enterprise**: Wait for v1.0-beta for full web platform deployment

## Support & Feedback

### Issue Reporting
- **Bugs**: Report via GitHub Issues
- **Security**: Contact security@uveddi.dev
- **Feature Requests**: GitHub Discussions

### Community
- **Discord**: Community support (coming soon)
- **Documentation**: Comprehensive guides in `/docs/user-guide/`
- **Examples**: Sample configurations in `/docs/examples/`

### Version Support Policy
- **Current Alpha**: Active development, regular updates
- **Previous Alpha**: Security fixes only
- **GA Releases**: Long-term support (TBD)

---

## Document Maintenance

These release notes are maintained by the Uveddi development team and updated with each release. For the most current information, always refer to the latest version in this directory.

**Last Updated**: January 2025  
**Next Review**: With each release  
**Maintainers**: Uveddi Product & Engineering Teams