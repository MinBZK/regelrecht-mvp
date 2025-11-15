"""
API Router

REST endpoints for the RegelRecht demo backend.
"""

from fastapi import APIRouter, HTTPException
from uuid import UUID
from typing import List

from backend.models.law import Law, LawSummary, ArticleWithId
from backend.services.yaml_loader import get_yaml_loader


router = APIRouter(prefix="/api", tags=["laws"])


@router.get("/laws", response_model=List[LawSummary])
async def get_all_laws():
    """
    Get summaries of all available laws.

    Returns:
        List of law summaries with basic metadata
    """
    loader = get_yaml_loader()
    return loader.get_all_law_summaries()


@router.get("/laws/{law_uuid}", response_model=Law)
async def get_law_by_id(law_uuid: UUID):
    """
    Get a specific law by its UUID.

    Args:
        law_uuid: The UUID of the law

    Returns:
        Complete law with all articles

    Raises:
        HTTPException: 404 if law not found
    """
    loader = get_yaml_loader()
    law = loader.get_law_by_uuid(law_uuid)

    if law is None:
        raise HTTPException(
            status_code=404,
            detail=f"Law with UUID {law_uuid} not found"
        )

    return law


@router.get("/laws/{law_uuid}/articles", response_model=List[ArticleWithId])
async def get_law_articles(law_uuid: UUID):
    """
    Get all articles for a specific law.

    Args:
        law_uuid: The UUID of the law

    Returns:
        List of articles with IDs

    Raises:
        HTTPException: 404 if law not found
    """
    loader = get_yaml_loader()
    law = loader.get_law_by_uuid(law_uuid)

    if law is None:
        raise HTTPException(
            status_code=404,
            detail=f"Law with UUID {law_uuid} not found"
        )

    # Add IDs to articles
    articles_with_ids = []
    for article in law.articles:
        article_data = article.model_dump()
        article_data["id"] = f"{law_uuid}:{article.number}"
        article_data["law_uuid"] = law_uuid
        articles_with_ids.append(ArticleWithId(**article_data))

    return articles_with_ids


@router.get("/laws/bwb/{bwb_id}", response_model=Law)
async def get_law_by_bwb_id(bwb_id: str):
    """
    Get a specific law by its BWB ID.

    Args:
        bwb_id: The BWB identification number (e.g., BWBR0018451)

    Returns:
        Complete law with all articles

    Raises:
        HTTPException: 404 if law not found
    """
    loader = get_yaml_loader()
    law = loader.get_law_by_bwb_id(bwb_id)

    if law is None:
        raise HTTPException(
            status_code=404,
            detail=f"Law with BWB ID {bwb_id} not found"
        )

    return law


@router.get("/health")
async def health_check():
    """
    Health check endpoint.

    Returns:
        Status information
    """
    loader = get_yaml_loader()
    laws = loader.get_all_laws()

    return {
        "status": "healthy",
        "laws_loaded": len(laws),
        "version": "0.1.0"
    }
