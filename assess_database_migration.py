#!/usr/bin/env python3
"""
assess_database_migration.py - Analyze the impact of extracting database module
"""

import os
import re
import json
from pathlib import Path
from dataclasses import dataclass, field
from typing import List, Set, Dict, Tuple
import subprocess

@dataclass
class FileImpact:
    path: str
    imports: List[str] = field(default_factory=list)
    database_uses: List[str] = field(default_factory=list)
    types_used: Set[str] = field(default_factory=set)
    line_count: int = 0
    complexity_score: int = 0

@dataclass
class MigrationAssessment:
    total_files_affected: int = 0
    database_module_size: int = 0
    external_dependencies: Dict[str, int] = field(default_factory=dict)
    types_to_export: Set[str] = field(default_factory=set)
    breaking_changes: List[str] = field(default_factory=list)
    migration_steps: List[str] = field(default_factory=list)

class DatabaseMigrationAnalyzer:
    def __init__(self, project_root: str):
        self.project_root = Path(project_root)
        self.src_dir = self.project_root / "src"
        self.database_types = {
            'Database', 'Container', 'Block', 'BlockRelationship',
            'BlockVersion', 'PromptTemplate', 'LLMInteraction',
            'SemanticBranch', 'EnhancedBlockRelationship',
            'BlockTemplate', 'ReconstructionMetadata',
            'BlockEvolution', 'SourceCodeMigrationLog',
            'SourceCodeBackup', 'SourceCodeMigrator'
        }
        self.impacts: Dict[str, FileImpact] = {}
        
    def analyze(self) -> MigrationAssessment:
        assessment = MigrationAssessment()
        
        # 1. Analyze database module size
        assessment.database_module_size = self._analyze_database_module()
        
        # 2. Find all files using database
        self._find_database_usage(assessment)
        
        # 3. Analyze type dependencies
        self._analyze_type_dependencies(assessment)
        
        # 4. Identify breaking changes
        self._identify_breaking_changes(assessment)
        
        # 5. Generate migration plan
        self._generate_migration_plan(assessment)
        
        return assessment
    
    def _analyze_database_module(self) -> int:
        """Analyze the current database module size"""
        total_lines = 0
        db_path = self.src_dir / "database"
        
        if db_path.exists():
            for file in db_path.glob("*.rs"):
                with open(file, 'r') as f:
                    lines = f.readlines()
                    total_lines += len(lines)
                    
        return total_lines
    
    def _find_database_usage(self, assessment: MigrationAssessment):
        """Find all files importing from database module"""
        for rust_file in self.src_dir.rglob("*.rs"):
            if "database" in str(rust_file):
                continue
                
            impact = self._analyze_file(rust_file)
            if impact.imports or impact.database_uses:
                self.impacts[str(rust_file)] = impact
                assessment.total_files_affected += 1
                
    def _analyze_file(self, file_path: Path) -> FileImpact:
        """Analyze a single Rust file for database usage"""
        impact = FileImpact(path=str(file_path.relative_to(self.project_root)))
        
        try:
            with open(file_path, 'r') as f:
                content = f.read()
                lines = content.split('\n')
                impact.line_count = len(lines)
                
                # Find database imports
                import_patterns = [
                    r'use\s+crate::database::\{([^}]+)\}',
                    r'use\s+crate::database::([^;]+);',
                    r'use\s+super::database::([^;]+);',
                    r'use\s+metaforge_engine::database::([^;]+);',
                    r'Database::new',
                    r'Database::setup',
                ]
                
                for pattern in import_patterns:
                    matches = re.findall(pattern, content)
                    for match in matches:
                        impact.imports.append(match)
                        
                # Find database type usage
                for db_type in self.database_types:
                    if db_type in content:
                        impact.types_used.add(db_type)
                        # Count occurrences
                        count = content.count(db_type)
                        impact.database_uses.append(f"{db_type}: {count}")
                        
                # Calculate complexity score
                impact.complexity_score = self._calculate_complexity(content)
                
        except Exception as e:
            print(f"Error analyzing {file_path}: {e}")
            
        return impact
    
    def _calculate_complexity(self, content: str) -> int:
        """Simple complexity score based on database usage patterns"""
        score = 0
        
        # Direct database operations
        if 'db.' in content or 'database.' in content:
            score += content.count('db.') + content.count('database.')
            
        # Transaction usage
        if 'transaction' in content.lower():
            score += 10
            
        # Complex queries
        if 'sqlx::query' in content:
            score += 5 * content.count('sqlx::query')
            
        # Database pool usage
        if 'PgPool' in content:
            score += 3
            
        # Async database calls
        if '.await?' in content:
            score += content.count('.await?')
            
        return score
    
    def _analyze_type_dependencies(self, assessment: MigrationAssessment):
        """Identify all types that need to be exported"""
        for impact in self.impacts.values():
            assessment.types_to_export.update(impact.types_used)
            
        # Add indirect dependencies
        if 'Block' in assessment.types_to_export:
            assessment.types_to_export.add('BlockRelationship')
        if 'Database' in assessment.types_to_export:
            assessment.types_to_export.update(['Container', 'Block'])
            
    def _identify_breaking_changes(self, assessment: MigrationAssessment):
        """Identify potential breaking changes"""
        
        # Check for direct field access
        for impact in self.impacts.values():
            if impact.complexity_score > 20:
                assessment.breaking_changes.append(
                    f"High coupling in {impact.path} (score: {impact.complexity_score})"
                )
                
        # Check for modules that directly construct database types
        high_impact_files = [
            "src/main.rs",
            "src/ai_operations/",
            "src/versioning/",
            "src/analysis/",
            "src/generator/"
        ]
        
        for impact in self.impacts.values():
            for high_impact in high_impact_files:
                if high_impact in impact.path and impact.complexity_score > 10:
                    assessment.breaking_changes.append(
                        f"Critical dependency in {impact.path} - requires careful migration"
                    )
                    break
                    
    def _generate_migration_plan(self, assessment: MigrationAssessment):
        """Generate step-by-step migration plan"""
        assessment.migration_steps = [
            "1. Create metaforge-database crate structure with Cargo.toml",
            "2. Move database module files (schema.rs, source_code_migrator.rs) to new crate",
            "3. Extract shared types (Database, Container, Block) to metaforge-core",
            "4. Update workspace Cargo.toml with new crates and dependencies",
            "5. Create trait definitions in metaforge-core for database operations",
            "6. Update all 35+ files with new import paths",
            "7. Create compatibility re-exports in main crate for smooth transition",
            "8. Update binary crates (test_versioning, phase2_executor, test_migrations)",
            "9. Test each affected module systematically",
            "10. Remove old database module and compatibility layer"
        ]
        
    def generate_report(self, assessment: MigrationAssessment) -> str:
        """Generate detailed migration report"""
        report = []
        report.append("=" * 80)
        report.append("DATABASE MODULE MIGRATION ASSESSMENT")
        report.append("MetaForge Engine - First Move Analysis")
        report.append("=" * 80)
        
        report.append(f"\n📊 CRITICAL METRICS:")
        report.append(f"  • Database module size: {assessment.database_module_size:,} lines")
        report.append(f"  • Files affected: {assessment.total_files_affected}/72 total Rust files ({assessment.total_files_affected/72*100:.1f}%)")
        report.append(f"  • Types to export: {len(assessment.types_to_export)}")
        report.append(f"  • Potential breaking changes: {len(assessment.breaking_changes)}")
        
        # Risk assessment
        risk_level = "HIGH" if assessment.total_files_affected > 30 else \
                     "MEDIUM" if assessment.total_files_affected > 15 else "LOW"
        
        report.append(f"\n⚠️  RISK ASSESSMENT: {risk_level}")
        if risk_level == "HIGH":
            report.append(f"  🚨 CRITICAL: {assessment.total_files_affected} files affected - requires comprehensive testing")
        elif risk_level == "MEDIUM":
            report.append(f"  ⚠️  MODERATE: Significant impact but manageable with proper planning")
        else:
            report.append(f"  ✅ LOW: Limited impact, safe to proceed")
        
        report.append(f"\n🔍 AFFECTED FILES BY COMPLEXITY:")
        sorted_impacts = sorted(
            self.impacts.items(), 
            key=lambda x: x[1].complexity_score, 
            reverse=True
        )
        
        for path, impact in sorted_impacts[:15]:
            report.append(f"  {impact.complexity_score:3d} - {path}")
            if impact.types_used:
                types_str = ', '.join(list(impact.types_used)[:3])
                if len(impact.types_used) > 3:
                    types_str += f" +{len(impact.types_used)-3} more"
                report.append(f"       Uses: {types_str}")
                
        report.append(f"\n⚠️  BREAKING CHANGES:")
        for change in assessment.breaking_changes:
            report.append(f"  • {change}")
            
        report.append(f"\n📦 TYPES TO EXPORT ({len(assessment.types_to_export)}):")
        types_list = sorted(assessment.types_to_export)
        for i in range(0, len(types_list), 3):
            chunk = types_list[i:i+3]
            report.append(f"  {', '.join(chunk)}")
                
        report.append(f"\n📝 MIGRATION PLAN:")
        for step in assessment.migration_steps:
            report.append(f"  {step}")
            
        # Specific recommendations based on analysis
        report.append(f"\n🎯 SPECIFIC RECOMMENDATIONS:")
        if assessment.total_files_affected > 30:
            report.append("  • Create comprehensive integration tests before migration")
            report.append("  • Consider phased migration approach")
            report.append("  • Implement feature flags for rollback capability")
        
        report.append("  • Focus on main.rs first - it has the highest database usage")
        report.append("  • Create metaforge-core crate for shared types")
        report.append("  • Use workspace dependencies to avoid version conflicts")
        report.append("  • Keep compatibility layer during transition period")
        
        return "\n".join(report)
    
    def generate_migration_script(self) -> str:
        """Generate shell script for migration"""
        script = []
        script.append("#!/bin/bash")
        script.append("set -e")
        script.append("")
        script.append("# Database Module Migration Script")
        script.append("# Generated by assess_database_migration.py")
        script.append("# MetaForge Engine - First Move Implementation")
        script.append("")
        
        script.append("echo '🚀 Starting database module migration...'")
        script.append("")
        
        # Create workspace structure
        script.append("# Step 1: Create workspace structure")
        script.append("mkdir -p crates")
        script.append("")
        
        # Create metaforge-core crate
        script.append("# Step 2: Create metaforge-core crate for shared types")
        script.append("cargo new --lib crates/metaforge-core")
        script.append("")
        
        # Create metaforge-database crate
        script.append("# Step 3: Create metaforge-database crate")
        script.append("cargo new --lib crates/metaforge-database")
        script.append("")
        
        # Copy database files
        script.append("# Step 4: Copy database files")
        script.append("cp src/database/schema.rs crates/metaforge-database/src/")
        script.append("cp src/database/source_code_migrator.rs crates/metaforge-database/src/")
        script.append("cp -r migrations/ crates/metaforge-database/")
        script.append("")
        
        # Update workspace Cargo.toml
        script.append("# Step 5: Update workspace Cargo.toml")
        script.append("cat > Cargo.toml << 'EOF'")
        script.append("[workspace]")
        script.append("members = [")
        script.append('    "crates/metaforge-core",')
        script.append('    "crates/metaforge-database",')
        script.append('    ".",  # Keep root crate')
        script.append("]")
        script.append("")
        script.append("[workspace.dependencies]")
        script.append('sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "uuid", "chrono", "json"] }')
        script.append('uuid = { version = "1.6", features = ["serde", "v4"] }')
        script.append('serde = { version = "1.0", features = ["derive"] }')
        script.append('chrono = { version = "0.4", features = ["serde"] }')
        script.append('anyhow = "1.0"')
        script.append('tokio = { version = "1.35", features = ["full"] }')
        script.append("")
        script.append("[package]")
        script.append('name = "metaforge-engine"')
        script.append('version = "0.1.0"')
        script.append('edition = "2021"')
        script.append("")
        script.append("[dependencies]")
        script.append("metaforge-core = { path = \"crates/metaforge-core\" }")
        script.append("metaforge-database = { path = \"crates/metaforge-database\" }")
        script.append("# ... other existing dependencies")
        script.append("EOF")
        script.append("")
        
        # Create metaforge-core lib.rs
        script.append("# Step 6: Create metaforge-core lib.rs")
        script.append("cat > crates/metaforge-core/src/lib.rs << 'EOF'")
        script.append("//! MetaForge Core - Shared types and traits")
        script.append("")
        script.append("pub mod semantic_block;")
        script.append("pub mod database_traits;")
        script.append("")
        script.append("// Re-export key types")
        script.append("pub use semantic_block::*;")
        script.append("pub use database_traits::*;")
        script.append("EOF")
        script.append("")
        
        # Create metaforge-database lib.rs
        script.append("# Step 7: Create metaforge-database lib.rs")
        script.append("cat > crates/metaforge-database/src/lib.rs << 'EOF'")
        script.append("//! MetaForge Database - Database layer implementation")
        script.append("")
        script.append("pub mod schema;")
        script.append("pub mod source_code_migrator;")
        script.append("")
        script.append("// Re-export key types")
        script.append("pub use schema::*;")
        script.append("pub use source_code_migrator::*;")
        script.append("EOF")
        script.append("")
        
        # Create compatibility bridge
        script.append("# Step 8: Create compatibility bridge")
        script.append("cat > src/database_bridge.rs << 'EOF'")
        script.append("//! Temporary bridge module for database migration")
        script.append("//! This will be removed after migration is complete")
        script.append("")
        script.append("// Re-export from new crates")
        script.append("pub use metaforge_database::{")
        script.append("    Database, Container, Block, BlockRelationship,")
        script.append("    SourceCodeMigrator")
        script.append("};")
        script.append("")
        script.append("// Re-export schema module")
        script.append("pub mod schema {")
        script.append("    pub use metaforge_database::*;")
        script.append("}")
        script.append("EOF")
        script.append("")
        
        # Update main lib.rs
        script.append("# Step 9: Update main lib.rs")
        script.append("sed -i 's/pub mod database;/pub mod database_bridge as database;/' src/lib.rs")
        script.append("")
        
        # Test compilation
        script.append("# Step 10: Test compilation")
        script.append("echo '🧪 Testing compilation...'")
        script.append("cargo check --all")
        script.append("")
        
        # Run tests
        script.append("# Step 11: Run tests")
        script.append("echo '🧪 Running tests...'")
        script.append("cargo test --all")
        script.append("")
        
        script.append("echo '✅ Database module migration complete!'")
        script.append("echo '📝 Next steps:'")
        script.append("echo '  1. Update import paths in affected files'")
        script.append("echo '  2. Remove database_bridge.rs when ready'")
        script.append("echo '  3. Update documentation'")
        
        return "\n".join(script)

def main():
    import sys
    
    project_root = sys.argv[1] if len(sys.argv) > 1 else "."
    
    print("🔍 Analyzing MetaForge Engine database migration impact...")
    print("=" * 60)
    
    analyzer = DatabaseMigrationAnalyzer(project_root)
    assessment = analyzer.analyze()
    
    # Generate and print report
    print(analyzer.generate_report(assessment))
    
    # Save detailed analysis
    analysis_data = {
        "total_files_affected": assessment.total_files_affected,
        "database_module_size": assessment.database_module_size,
        "types_to_export": list(assessment.types_to_export),
        "breaking_changes": assessment.breaking_changes,
        "migration_steps": assessment.migration_steps,
        "file_impacts": {
            path: {
                "complexity_score": impact.complexity_score,
                "types_used": list(impact.types_used),
                "line_count": impact.line_count,
                "imports": impact.imports
            }
            for path, impact in analyzer.impacts.items()
        },
        "risk_assessment": {
            "level": "HIGH" if assessment.total_files_affected > 30 else 
                     "MEDIUM" if assessment.total_files_affected > 15 else "LOW",
            "affected_percentage": assessment.total_files_affected / 72 * 100,
            "recommendation": "Create comprehensive tests first" if assessment.total_files_affected > 30 else "Proceed with caution"
        }
    }
    
    with open("database_migration_analysis.json", "w") as f:
        json.dump(analysis_data, f, indent=2)
    
    # Generate migration script
    with open("migrate_database.sh", "w") as f:
        f.write(analyzer.generate_migration_script())
    
    # Make script executable
    os.chmod("migrate_database.sh", 0o755)
    
    print("\n📄 Files generated:")
    print("  • database_migration_analysis.json - Detailed impact analysis")
    print("  • migrate_database.sh - Executable migration script")
    
    # Final risk assessment
    risk_level = analysis_data["risk_assessment"]["level"]
    affected_pct = analysis_data["risk_assessment"]["affected_percentage"]
    
    print(f"\n🎯 FINAL ASSESSMENT:")
    print(f"  Risk Level: {risk_level}")
    print(f"  Files Affected: {assessment.total_files_affected}/72 ({affected_pct:.1f}%)")
    print(f"  Database Module Size: {assessment.database_module_size:,} lines")
    
    if risk_level == "HIGH":
        print(f"\n🚨 HIGH RISK MIGRATION:")
        print(f"  • {assessment.total_files_affected} files need import updates")
        print(f"  • Comprehensive testing required before proceeding")
        print(f"  • Consider phased migration approach")
        print(f"  • Estimated effort: 2-3 days with testing")
    elif risk_level == "MEDIUM":
        print(f"\n⚠️  MEDIUM RISK MIGRATION:")
        print(f"  • Manageable scope with proper planning")
        print(f"  • Focus on high-complexity files first")
        print(f"  • Estimated effort: 1-2 days")
    else:
        print(f"\n✅ LOW RISK MIGRATION:")
        print(f"  • Safe to proceed with standard precautions")
        print(f"  • Estimated effort: 4-8 hours")
    
    print(f"\n🚀 Ready to execute? Run: ./migrate_database.sh")

if __name__ == "__main__":
    main()
