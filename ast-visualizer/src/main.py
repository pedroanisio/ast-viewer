#!/usr/bin/env python3
"""
GenAI-Enhanced Code Inspector CLI

A sophisticated tool that combines pattern matching with AI-powered analysis
for intelligent functional requirement extraction from codebases.
"""

import asyncio
import os
import sys
from pathlib import Path

try:
    import click
    from dotenv import load_dotenv
except ImportError:
    print("Missing dependencies. Please install: pip install click python-dotenv")
    sys.exit(1)

# Add the current directory to Python path
sys.path.insert(0, str(Path(__file__).parent))

from code_inspector import CodeInspector, InspectorConfig
from code_inspector.config.settings import AnalysisMode, OpenAIModel
from code_inspector.utils.output_formatter import OutputFormatter

# Load environment variables
load_dotenv()


@click.group()
@click.version_option(version="2.0.0")
def cli():
    """GenAI-Enhanced Code Inspector - Extract functional requirements from codebases using AI"""
    pass


@cli.command()
@click.argument('codebase_path', type=click.Path(exists=True, file_okay=False, dir_okay=True))
@click.option('-o', '--output', default='requirements.json', 
              help='Output file path (format auto-detected from extension)')
@click.option('--mode', type=click.Choice(['pattern_only', 'ai_only', 'hybrid', 'ai_validation']),
              default='hybrid', help='Analysis mode')
@click.option('--model', type=click.Choice(['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo', 'gpt-3.5-turbo']),
              default='gpt-4o-mini', help='OpenAI model to use')
@click.option('--api-key', help='OpenAI API key (or set OPENAI_API_KEY env var)')
@click.option('--max-workers', type=int, default=4, help='Maximum worker threads')
@click.option('--min-confidence', type=float, default=0.6, 
              help='Minimum confidence threshold (0.0-1.0)')
@click.option('--no-parallel', is_flag=True, help='Disable parallel processing')
@click.option('--no-progress', is_flag=True, help='Disable progress bars')
@click.option('--no-snippets', is_flag=True, help='Exclude code snippets from output')
@click.option('--debug', is_flag=True, help='Enable debug logging')
@click.option('--config-file', type=click.Path(exists=True), 
              help='Load configuration from JSON file')
def analyze(codebase_path, output, mode, model, api_key, max_workers, min_confidence,
           no_parallel, no_progress, no_snippets, debug, config_file):
    """Analyze a codebase and extract functional requirements"""
    
    try:
        # Create configuration
        config = _create_config(
            mode=mode,
            model=model,
            api_key=api_key,
            max_workers=max_workers,
            min_confidence=min_confidence,
            enable_parallel=not no_parallel,
            show_progress=not no_progress,
            include_code_snippets=not no_snippets,
            debug=debug,
            config_file=config_file
        )
        
        # Validate configuration
        if config.requires_ai() and not config.openai_api_key:
            click.echo("❌ Error: OpenAI API key required for AI analysis modes")
            click.echo("Set OPENAI_API_KEY environment variable or use --api-key option")
            sys.exit(1)
        
        # Run analysis
        asyncio.run(_run_analysis(codebase_path, output, config))
        
    except Exception as e:
        click.echo(f"❌ Analysis failed: {e}")
        if debug:
            import traceback
            traceback.print_exc()
        sys.exit(1)


@cli.command()
@click.argument('codebase_path', type=click.Path(exists=True, file_okay=False, dir_okay=True))
@click.option('--output-dir', default='./demo_results', help='Output directory for demo results')
def demo(codebase_path, output_dir):
    """Run a comprehensive demo analysis with multiple formats and modes"""
    
    click.echo("🚀 Running Code Inspector Demo...")
    
    # Create output directory
    output_path = Path(output_dir)
    output_path.mkdir(exist_ok=True)
    
    # Demo configurations
    demo_configs = [
        {
            'name': 'Pattern Only',
            'mode': AnalysisMode.PATTERN_ONLY,
            'output': 'pattern_only_results.json'
        },
        {
            'name': 'AI Analysis (if API key available)',
            'mode': AnalysisMode.HYBRID,
            'output': 'hybrid_results.json'
        }
    ]
    
    for demo_config in demo_configs:
        try:
            click.echo(f"\n📊 Running {demo_config['name']}...")
            
            config = InspectorConfig(
                analysis_mode=demo_config['mode'],
                show_progress=True,
                include_code_snippets=True
            )
            
            # Skip AI modes if no API key
            if config.requires_ai() and not config.openai_api_key:
                click.echo("⚠️  Skipping AI analysis (no API key provided)")
                continue
            
            output_file = output_path / demo_config['output']
            asyncio.run(_run_analysis(codebase_path, str(output_file), config))
            
            # Also save as HTML and Markdown
            base_name = output_file.stem
            asyncio.run(_run_analysis(codebase_path, str(output_path / f"{base_name}.html"), config))
            asyncio.run(_run_analysis(codebase_path, str(output_path / f"{base_name}.md"), config))
            
        except Exception as e:
            click.echo(f"❌ Demo {demo_config['name']} failed: {e}")
    
    click.echo(f"\n✅ Demo complete! Results saved to: {output_path}")


@cli.command()
def test_connection():
    """Test OpenAI API connection"""
    
    api_key = os.getenv('OPENAI_API_KEY')
    if not api_key:
        click.echo("❌ No OpenAI API key found")
        click.echo("Set OPENAI_API_KEY environment variable")
        return
    
    try:
        from code_inspector.ai_analyzer.openai_analyzer import OpenAIAnalyzer
        from code_inspector.config.settings import InspectorConfig
        
        config = InspectorConfig(openai_api_key=api_key)
        analyzer = OpenAIAnalyzer(config)
        
        # Test with a simple code snippet
        test_files = [{
            'path': 'test.py',
            'content': 'def hello_world():\n    """Say hello"""\n    return "Hello, World!"',
            'functions': [{'name': 'hello_world', 'docstring': 'Say hello'}],
            'classes': [],
            'imports': []
        }]
        
        result = asyncio.run(analyzer.analyze_code_files(test_files))
        
        click.echo(f"✅ OpenAI connection successful!")
        click.echo(f"📊 Test analysis found {len(result)} requirements")
        
        stats = analyzer.get_usage_stats()
        click.echo(f"🔧 Model: {stats['model_used']}")
        click.echo(f"⚡ Tokens used: {stats['total_tokens_used']}")
        
    except Exception as e:
        click.echo(f"❌ OpenAI connection failed: {e}")


@cli.command()
@click.argument('input_file', type=click.Path(exists=True))
@click.argument('output_file', type=click.Path())
@click.option('--format', type=click.Choice(['json', 'yaml', 'html', 'markdown', 'csv']),
              help='Output format (auto-detected if not specified)')
def convert(input_file, output_file, format):
    """Convert analysis results between different formats"""
    
    try:
        import json
        
        # Load results
        with open(input_file, 'r', encoding='utf-8') as f:
            data = json.load(f)
        
        # Convert to AnalysisResult (simplified)
        click.echo(f"🔄 Converting {input_file} to {output_file}...")
        
        # This would need proper deserialization from JSON back to Pydantic models
        # For now, just copy the file with format conversion
        
        if format == 'yaml' or output_file.endswith('.yaml') or output_file.endswith('.yml'):
            import yaml
            with open(output_file, 'w', encoding='utf-8') as f:
                yaml.dump(data, f, default_flow_style=False)
        else:
            # Default to JSON
            with open(output_file, 'w', encoding='utf-8') as f:
                json.dump(data, f, indent=2, ensure_ascii=False)
        
        click.echo("✅ Conversion complete!")
        
    except Exception as e:
        click.echo(f"❌ Conversion failed: {e}")


def _create_config(**kwargs) -> InspectorConfig:
    """Create configuration from CLI arguments"""
    
    # Load from config file if provided
    if kwargs.get('config_file'):
        import json
        with open(kwargs['config_file'], 'r') as f:
            file_config = json.load(f)
        config = InspectorConfig(**file_config)
    else:
        config = InspectorConfig()
    
    # Override with CLI arguments
    if kwargs.get('mode'):
        config.analysis_mode = AnalysisMode(kwargs['mode'])
    
    if kwargs.get('model'):
        config.ai_model = OpenAIModel(kwargs['model'])
    
    if kwargs.get('api_key'):
        config.openai_api_key = kwargs['api_key']
    
    if kwargs.get('max_workers'):
        config.max_workers = kwargs['max_workers']
    
    if kwargs.get('min_confidence'):
        config.min_confidence_threshold = kwargs['min_confidence']
    
    if 'enable_parallel' in kwargs:
        config.enable_parallel = kwargs['enable_parallel']
    
    if 'show_progress' in kwargs:
        config.show_progress = kwargs['show_progress']
    
    if 'include_code_snippets' in kwargs:
        config.include_code_snippets = kwargs['include_code_snippets']
    
    if kwargs.get('debug'):
        config.log_level = "DEBUG"
    
    return config


async def _run_analysis(codebase_path: str, output_path: str, config: InspectorConfig):
    """Run the analysis with the given configuration"""
    
    # Initialize inspector
    inspector = CodeInspector(codebase_path, config)
    
    # Run analysis
    result = await inspector.analyze_codebase()
    
    # Save results
    await inspector.save_results(result, output_path)
    
    # Print summary
    formatter = OutputFormatter(config)
    formatter.print_summary(result)
    
    # Print statistics
    stats = inspector.get_stats()
    if stats.get('total_tokens_used', 0) > 0:
        print(f"\n💰 AI Usage: {stats['total_tokens_used']} tokens, ~${stats.get('estimated_cost', 0):.4f}")


if __name__ == '__main__':
    cli()
