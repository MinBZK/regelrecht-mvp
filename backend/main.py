"""
RegelRecht Demo Backend

FastAPI backend for the Blockly editor demo.
"""

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from pathlib import Path
from contextlib import asynccontextmanager

from backend.routers import api
from backend.services.yaml_loader import init_yaml_loader


@asynccontextmanager
async def lifespan(app: FastAPI):
    """
    Initialize and cleanup services using modern lifespan handler.
    """
    # Startup: Initialize services
    backend_dir = Path(__file__).parent
    project_root = backend_dir.parent
    regulation_dir = project_root / "regulation" / "nl"

    if not regulation_dir.exists():
        print(f"Warning: Regulation directory not found at {regulation_dir}")
        print("Creating empty regulation directory...")
        regulation_dir.mkdir(parents=True, exist_ok=True)
    else:
        print(f"Loading laws from: {regulation_dir}")

    # Initialize YAML loader
    init_yaml_loader(regulation_dir)
    print("YAML loader initialized successfully")

    yield

    # Shutdown: Cleanup (if needed)
    print("Shutting down...")


# Initialize FastAPI app with lifespan
app = FastAPI(
    title="RegelRecht Demo API",
    description="Backend API for the RegelRecht Blockly Editor Demo",
    version="0.1.0",
    docs_url="/api/docs",
    redoc_url="/api/redoc",
    lifespan=lifespan
)

# Configure CORS for local development
app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://localhost:5173",  # Vite default dev server
        "http://localhost:3000",  # Alternative React dev server
        "http://127.0.0.1:5173",
        "http://127.0.0.1:3000",
    ],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Include API router
app.include_router(api.router)


@app.get("/")
async def root():
    """Root endpoint"""
    return {
        "message": "RegelRecht Demo API",
        "version": "0.1.0",
        "docs": "/api/docs"
    }


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(
        "backend.main:app",
        host="0.0.0.0",
        port=8000,
        reload=True  # Enable auto-reload for development
    )
