/**
 * Get all models for a session
 * @param {string} sessionId - Session ID
 * @returns {Promise<Array>} List of models
 */
export async function getSessionModels(sessionId) {
  const response = await fetch(`${API_BASE_URL}/sessions/${sessionId}/models`, {
    method: 'GET',
    headers: {
      Accept: 'application/json',
    },
  });
  return handleResponse(response);
}
/**
 * API Client for communicating with the backend service
 */

const API_BASE_URL = 'http://127.0.0.1:3000/api';

/**
 * Custom error for API responses
 */
export class ApiError extends Error {
  constructor(code, message, details = null) {
    super(message);
    this.name = 'ApiError';
    this.code = code;
    this.details = details;
  }
}

/**
 * Handle API response and parse JSON
 * @param {Response} response - Fetch response object
 * @returns {Promise<any>} Parsed JSON response
 */
async function handleResponse(response) {
  const contentType = response.headers.get('content-type');

  if (!response.ok) {
    if (contentType && contentType.includes('application/json')) {
      const errorData = await response.json();
      throw new ApiError(
        errorData.error?.code || 'UNKNOWN_ERROR',
        errorData.error?.message || 'Une erreur est survenue',
        errorData.error?.details
      );
    }
    throw new ApiError('HTTP_ERROR', `HTTP ${response.status}: ${response.statusText}`);
  }

  if (response.status === 204) {
    return null;
  }

  if (contentType && contentType.includes('application/json')) {
    return response.json();
  }

  return response.text();
}

/**
 * Create a new quote session
 * @returns {Promise<{session_id: string, expires_at: string}>}
 */
export async function createSession() {
  const response = await fetch(`${API_BASE_URL}/sessions`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
  });
  return handleResponse(response);
}

/**
 * Upload a 3D model file
 * @param {string} sessionId - Session ID
 * @param {File} file - File to upload
 * @param {function} onProgress - Progress callback (not supported with fetch)
 * @returns {Promise<Object>} Upload result with model info
 */
export async function uploadModel(sessionId, file, onProgress = null) {
  const formData = new FormData();
  formData.append('file', file);

  // Notify progress start (fetch doesn't support progress tracking)
  if (onProgress) {
    onProgress(10); // Indicate upload started
  }

  try {
    const response = await fetch(`${API_BASE_URL}/sessions/${sessionId}/models`, {
      method: 'POST',
      body: formData,
      // Important: Do NOT set Content-Type header - browser will set it with boundary
      cache: 'no-cache',
      credentials: 'omit',
      mode: 'cors',
    });

    if (onProgress) {
      onProgress(90); // Indicate upload complete, processing
    }

    if (!response.ok) {
      const contentType = response.headers.get('content-type');
      if (contentType && contentType.includes('application/json')) {
        const errorData = await response.json();
        throw new ApiError(
          errorData.error?.code || 'UPLOAD_ERROR',
          errorData.error?.message || 'Upload failed',
          errorData.error?.details
        );
      }
      throw new ApiError('UPLOAD_ERROR', `Upload failed: ${response.status}`);
    }

    const data = await response.json();

    if (onProgress) {
      onProgress(100); // Complete
    }

    return data;
  } catch (error) {
    if (error instanceof ApiError) {
      throw error;
    }
    throw new ApiError('NETWORK_ERROR', error.message || 'Network error during upload');
  }
}

/**
 * Delete an uploaded model
 * @param {string} sessionId - Session ID
 * @param {string} modelId - Model ID to delete
 * @returns {Promise<void>}
 */
export async function deleteModel(sessionId, modelId) {
  const response = await fetch(`${API_BASE_URL}/sessions/${sessionId}/models/${modelId}`, {
    method: 'DELETE',
  });
  return handleResponse(response);
}

/**
 * Get available materials
 * @param {string} serviceType - Service type filter (optional)
 * @returns {Promise<Array>} List of materials
 */
export async function getMaterials(serviceType = '3d_printing') {
  const url = new URL(`${API_BASE_URL}/materials`);
  if (serviceType) {
    url.searchParams.set('service_type', serviceType);
  }

  const response = await fetch(url);
  return handleResponse(response);
}

/**
 * Configure a model with material selection
 * @param {string} sessionId - Session ID
 * @param {string} modelId - Model ID
 * @param {string} materialId - Material ID to assign
 * @returns {Promise<Object>} Configuration result with price estimate
 */
export async function configureModel(sessionId, modelId, materialId) {
  const response = await fetch(`${API_BASE_URL}/sessions/${sessionId}/models/${modelId}`, {
    method: 'PATCH',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ material_id: materialId }),
  });
  return handleResponse(response);
}

/**
 * Generate a final quote
 * @param {string} sessionId - Session ID
 * @returns {Promise<Object>} Quote with breakdown
 */
export async function generateQuote(sessionId) {
  const response = await fetch(`${API_BASE_URL}/sessions/${sessionId}/quote`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
  });
  return handleResponse(response);
}

/**
 * Get current quote calculation
 * @param {string} sessionId - Session ID
 * @returns {Promise<Object>} Current quote state
 */
export async function getCurrentQuote(sessionId) {
  const response = await fetch(`${API_BASE_URL}/sessions/${sessionId}/quote`);
  return handleResponse(response);
}

/**
 * Check API health
 * @returns {Promise<boolean>} True if API is healthy
 */
export async function checkHealth() {
  try {
    const response = await fetch(`${API_BASE_URL.replace('/api', '')}/health`);
    return response.ok;
  } catch (_e) {
    return false;
  }
}
