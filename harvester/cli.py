"""Command-line interface for the harvester."""

from datetime import datetime

import typer
from rich.console import Console

from harvester.config import validate_bwb_id, validate_date
from harvester.models import Law
from harvester.parsers.content_parser import download_content, parse_articles_split
from harvester.parsers.wti_parser import download_wti, parse_wti_metadata
from harvester.storage.yaml_writer import save_yaml

app = typer.Typer(
    name="harvester",
    help="Download Dutch legislation and convert to YAML format.",
)
console = Console()


@app.command()
def download(
    bwb_id: str = typer.Argument(
        ...,
        help="BWB identifier (e.g., BWBR0018451)",
    ),
    date: str | None = typer.Option(
        None,
        "--date",
        "-d",
        help="Effective date in YYYY-MM-DD format (default: today)",
    ),
) -> None:
    """Download a law by BWB ID and convert to YAML."""
    # Use today if no date provided
    effective_date = date or datetime.now().strftime("%Y-%m-%d")

    # Validate inputs before making HTTP requests
    validate_bwb_id(bwb_id)
    validate_date(effective_date)

    console.print(f"[bold]Downloading {bwb_id}[/bold] for date {effective_date}")
    console.print()

    try:
        # Download and parse WTI (metadata)
        console.print("[dim]Downloading WTI...[/dim]")
        wti_tree = download_wti(bwb_id)
        metadata = parse_wti_metadata(wti_tree)
        console.print(f"  Title: [green]{metadata.title}[/green]")
        console.print(f"  Type: {metadata.regulatory_layer.value}")

        # Download and parse content (legal text)
        console.print("[dim]Downloading content...[/dim]")
        content_tree = download_content(bwb_id, effective_date)
        articles = parse_articles_split(content_tree, bwb_id, effective_date)
        console.print(f"  Articles: {len(articles)}")

        # Create Law object
        law = Law(metadata=metadata, articles=articles)

        # Save to YAML
        console.print("[dim]Saving YAML...[/dim]")
        output_path = save_yaml(law, effective_date)
        console.print()
        console.print(f"[bold green]Saved to:[/bold green] {output_path}")

    except Exception as e:
        console.print(f"[bold red]Error:[/bold red] {e}")
        raise typer.Exit(1) from e


@app.command()
def version() -> None:
    """Show version information."""
    console.print("harvester 0.1.0")


def main() -> None:
    """Entry point for the CLI."""
    app()


if __name__ == "__main__":
    main()
