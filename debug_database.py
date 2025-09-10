#!/usr/bin/env python3
"""
Debug script to inspect database content and test single file generation
"""

import asyncio
import json
import sys
from pathlib import Path
import asyncpg
from typing import Dict, Any, List, Optional

# Database connection settings
DATABASE_URL = "postgresql://metaforge_user:metaforge_pass@localhost/metaforge"

async def connect_db():
    """Connect to the database"""
    try:
        conn = await asyncpg.connect(DATABASE_URL)
        print("✅ Connected to database")
        return conn
    except Exception as e:
        print(f"❌ Failed to connect to database: {e}")
        sys.exit(1)

async def inspect_containers(conn):
    """List all containers in the database"""
    print("\n🔍 Inspecting containers...")
    
    query = """
    SELECT id, name, language, original_path, created_at, migration_id
    FROM containers 
    ORDER BY created_at DESC 
    LIMIT 10
    """
    
    rows = await conn.fetch(query)
    
    if not rows:
        print("❌ No containers found in database")
        return None
    
    print(f"Found {len(rows)} containers:")
    for i, row in enumerate(rows):
        print(f"  {i+1}. {row['name']} ({row['language']}) - {row['original_path']}")
        print(f"     ID: {row['id']}")
        print(f"     Migration: {row['migration_id']}")
        print(f"     Created: {row['created_at']}")
        print()
    
    return rows

async def inspect_blocks_for_container(conn, container_id: str):
    """Inspect blocks for a specific container"""
    print(f"\n🔍 Inspecting blocks for container {container_id}...")
    
    query = """
    SELECT id, block_type, semantic_name, abstract_syntax, 
           syntax_preservation, semantic_metadata, position
    FROM blocks 
    WHERE container_id = $1
    ORDER BY position
    LIMIT 20
    """
    
    rows = await conn.fetch(query, container_id)
    
    if not rows:
        print("❌ No blocks found for this container")
        return []
    
    print(f"Found {len(rows)} blocks:")
    
    for i, row in enumerate(rows):
        print(f"\n  Block {i+1}: {row['semantic_name']} ({row['block_type']})")
        print(f"    ID: {row['id']}")
        print(f"    Position: {row['position']}")
        
        # Parse abstract_syntax JSON
        abstract_syntax = row['abstract_syntax']
        if abstract_syntax:
            # Handle both string and dict cases
            if isinstance(abstract_syntax, str):
                try:
                    abstract_syntax = json.loads(abstract_syntax)
                except json.JSONDecodeError:
                    print(f"    ❌ Failed to parse abstract_syntax JSON")
                    abstract_syntax = None
            
            if abstract_syntax and isinstance(abstract_syntax, dict):
                print(f"    Abstract Syntax Keys: {list(abstract_syntax.keys())}")
                
                # Check for our enhanced implementation data
                if 'implementation' in abstract_syntax:
                    impl_data = abstract_syntax['implementation']
                    print(f"    ✅ ENHANCED IMPLEMENTATION FOUND!")
                    print(f"       Keys: {list(impl_data.keys())}")
                    
                    if 'original_body' in impl_data:
                        original_body = impl_data['original_body']
                        print(f"       Original Body Preview: {repr(original_body[:100])}...")
                    
                    if 'variable_assignments' in impl_data:
                        var_assignments = impl_data['variable_assignments']
                        print(f"       Variable Assignments: {list(var_assignments.keys())}")
                else:
                    print(f"    ❌ No 'implementation' key in abstract_syntax")
            else:
                print(f"    ❌ abstract_syntax is not a valid dict")
        
        # Parse syntax_preservation JSON
        syntax_preservation = row['syntax_preservation']
        if syntax_preservation:
            # Handle both string and dict cases
            if isinstance(syntax_preservation, str):
                try:
                    syntax_preservation = json.loads(syntax_preservation)
                except json.JSONDecodeError:
                    print(f"    ❌ Failed to parse syntax_preservation JSON")
                    syntax_preservation = None
            
            if syntax_preservation and isinstance(syntax_preservation, dict):
                print(f"    Syntax Preservation Keys: {list(syntax_preservation.keys())}")
            else:
                print(f"    ❌ syntax_preservation is not a valid dict")
        
        print()
    
    return rows

async def test_single_file_generation(conn, container_id: str, target_file: str = "constants.py"):
    """Test generation for a single file"""
    print(f"\n🔨 Testing generation for {target_file} in container {container_id}...")
    
    # Get blocks for the specific file
    query = """
    SELECT c.original_path, b.id, b.block_type, b.semantic_name, b.abstract_syntax, b.position
    FROM blocks b
    JOIN containers c ON b.container_id = c.id
    WHERE b.container_id = $1 AND c.original_path LIKE $2
    ORDER BY b.position
    """
    
    rows = await conn.fetch(query, container_id, f"%{target_file}")
    
    if not rows:
        print(f"❌ No blocks found for {target_file}")
        return
    
    print(f"Found {len(rows)} blocks for {target_file}:")
    
    # Simulate code generation
    generated_code = []
    generated_code.append("# Generated Python code")
    generated_code.append("")
    
    for row in rows:
        block_type = row['block_type']
        semantic_name = row['semantic_name']
        abstract_syntax = row['abstract_syntax']
        
        # Handle JSON parsing
        if abstract_syntax and isinstance(abstract_syntax, str):
            try:
                abstract_syntax = json.loads(abstract_syntax)
            except json.JSONDecodeError:
                print(f"    ❌ Failed to parse abstract_syntax JSON")
                abstract_syntax = None
        
        print(f"  Processing: {semantic_name} ({block_type})")
        
        if block_type == 'Import':
            # Generate import
            if abstract_syntax and isinstance(abstract_syntax, dict) and 'module_path' in abstract_syntax:
                module_path = abstract_syntax['module_path']
                generated_code.append(f"import {module_path}")
            else:
                generated_code.append(f"# TODO: Import {semantic_name}")
        
        elif block_type == 'Variable':
            # Check for enhanced implementation data
            if abstract_syntax and isinstance(abstract_syntax, dict) and 'implementation' in abstract_syntax:
                impl_data = abstract_syntax['implementation']
                print(f"    ✅ Found implementation data: {list(impl_data.keys())}")
                
                if 'variable_assignments' in impl_data:
                    var_assignments = impl_data['variable_assignments']
                    if semantic_name in var_assignments:
                        assignment_info = var_assignments[semantic_name]
                        if 'literal_value' in assignment_info:
                            literal_value = assignment_info['literal_value']
                            
                            # Convert JSON value to Python literal
                            if isinstance(literal_value, str):
                                python_value = f'"{literal_value}"'
                            elif isinstance(literal_value, (int, float)):
                                python_value = str(literal_value)
                            elif isinstance(literal_value, bool):
                                python_value = "True" if literal_value else "False"
                            elif literal_value is None:
                                python_value = "None"
                            else:
                                python_value = str(literal_value)
                            
                            generated_code.append(f"{semantic_name} = {python_value}")
                            print(f"    ✅ Generated: {semantic_name} = {python_value}")
                        elif 'expression' in assignment_info:
                            expression = assignment_info['expression']
                            generated_code.append(f"{semantic_name} = {expression}")
                            print(f"    ✅ Generated: {semantic_name} = {expression}")
                        else:
                            generated_code.append(f"{semantic_name} = None  # No value preserved")
                            print(f"    ❌ No literal_value or expression found")
                    else:
                        generated_code.append(f"{semantic_name} = None  # Variable not in assignments")
                        print(f"    ❌ Variable {semantic_name} not found in assignments")
                else:
                    generated_code.append(f"{semantic_name} = None  # No variable_assignments")
                    print(f"    ❌ No variable_assignments in implementation data")
            else:
                generated_code.append(f"{semantic_name} = None  # No implementation data")
                print(f"    ❌ No implementation data found")
        
        elif block_type == 'Function':
            # Check for enhanced implementation data
            if abstract_syntax and isinstance(abstract_syntax, dict) and 'implementation' in abstract_syntax:
                impl_data = abstract_syntax['implementation']
                if 'original_body' in impl_data:
                    original_body = impl_data['original_body']
                    generated_code.append(f"def {semantic_name}():")
                    # Add indented body
                    for line in original_body.split('\n'):
                        if line.strip():
                            generated_code.append(f"    {line}")
                    print(f"    ✅ Generated function with preserved body")
                else:
                    generated_code.append(f"def {semantic_name}():")
                    generated_code.append(f"    pass  # No original body preserved")
                    print(f"    ❌ No original_body in implementation data")
            else:
                generated_code.append(f"def {semantic_name}():")
                generated_code.append(f"    pass  # No implementation data")
                print(f"    ❌ No implementation data found")
        
        generated_code.append("")
    
    # Write generated code to file
    output_file = f"debug_generated_{target_file}"
    with open(output_file, 'w') as f:
        f.write('\n'.join(generated_code))
    
    print(f"\n✅ Generated code written to: {output_file}")
    print("Generated code preview:")
    print("=" * 50)
    print('\n'.join(generated_code[:20]))
    if len(generated_code) > 20:
        print("... (truncated)")
    print("=" * 50)

async def main():
    """Main debug function"""
    print("🔍 MetaForge Database Debug Tool")
    print("=" * 50)
    
    conn = await connect_db()
    
    try:
        # 1. Inspect containers
        containers = await inspect_containers(conn)
        if not containers:
            return
        
        # 2. Let user choose a container or use the first one
        if len(sys.argv) > 1:
            container_index = int(sys.argv[1]) - 1
        else:
            container_index = 0
        
        if container_index >= len(containers):
            print(f"❌ Invalid container index. Available: 1-{len(containers)}")
            return
        
        selected_container = containers[container_index]
        container_id = str(selected_container['id'])
        
        print(f"\n🎯 Selected container: {selected_container['name']}")
        
        # 3. Inspect blocks for the selected container
        blocks = await inspect_blocks_for_container(conn, container_id)
        
        # 4. Test single file generation
        target_file = "constants.py" if len(sys.argv) <= 2 else sys.argv[2]
        await test_single_file_generation(conn, container_id, target_file)
        
    finally:
        await conn.close()
        print("\n✅ Database connection closed")

if __name__ == "__main__":
    print("Usage: python debug_database.py [container_number] [target_file]")
    print("Example: python debug_database.py 1 constants.py")
    print()
    
    asyncio.run(main())
