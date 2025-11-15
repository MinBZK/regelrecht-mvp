/**
 * API service layer for communicating with the FastAPI backend
 */

import axios, { AxiosError } from 'axios';
import type {
  Law,
  LawSummary,
  ArticleWithId,
  HealthResponse,
  ApiError,
} from '../types/schema';

// Create axios instance with default config
const api = axios.create({
  baseURL: '/api',
  headers: {
    'Content-Type': 'application/json',
  },
  timeout: 10000, // 10 second timeout
});

// Error handling helper
const handleApiError = (error: unknown): never => {
  if (axios.isAxiosError(error)) {
    const axiosError = error as AxiosError<ApiError>;
    const message = axiosError.response?.data?.detail || axiosError.message;
    throw new Error(message);
  }
  throw error;
};

/**
 * Health check endpoint
 */
export const checkHealth = async (): Promise<HealthResponse> => {
  try {
    const response = await api.get<HealthResponse>('/health');
    return response.data;
  } catch (error) {
    return handleApiError(error);
  }
};

/**
 * Get all laws with summary information
 */
export const getAllLaws = async (): Promise<LawSummary[]> => {
  try {
    const response = await api.get<LawSummary[]>('/laws');
    return response.data;
  } catch (error) {
    return handleApiError(error);
  }
};

/**
 * Get a specific law by UUID
 */
export const getLawByUuid = async (uuid: string): Promise<Law> => {
  try {
    const response = await api.get<Law>(`/laws/${uuid}`);
    return response.data;
  } catch (error) {
    return handleApiError(error);
  }
};

/**
 * Get articles from a law with generated IDs
 */
export const getLawArticles = async (uuid: string): Promise<ArticleWithId[]> => {
  try {
    const response = await api.get<ArticleWithId[]>(`/laws/${uuid}/articles`);
    return response.data;
  } catch (error) {
    return handleApiError(error);
  }
};

/**
 * Get a law by BWB ID (alternative lookup method)
 */
export const getLawByBwbId = async (bwbId: string): Promise<Law> => {
  try {
    const response = await api.get<Law>(`/laws/bwb/${bwbId}`);
    return response.data;
  } catch (error) {
    return handleApiError(error);
  }
};

/**
 * Update an article's machine-readable content (future feature)
 * This will be implemented when we add PATCH endpoints to the backend
 */
export const updateArticle = async (
  lawUuid: string,
  articleNumber: string,
  machineReadable: any
): Promise<void> => {
  try {
    // TODO: Implement when backend supports PATCH
    await api.patch(`/laws/${lawUuid}/articles/${articleNumber}`, {
      machine_readable: machineReadable,
    });
  } catch (error) {
    return handleApiError(error);
  }
};

export default api;
