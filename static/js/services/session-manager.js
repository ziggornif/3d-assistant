/**
 * Session Manager for managing user quote sessions
 */

import { createSession } from './api-client.js';

const SESSION_STORAGE_KEY = 'quote_session';

/**
 * Session data structure
 * @typedef {Object} SessionData
 * @property {string} sessionId - Session ID
 * @property {string} expiresAt - Expiration timestamp (ISO string)
 * @property {Array} models - Uploaded models
 */

class SessionManager {
  constructor() {
    this.sessionId = null;
    this.expiresAt = null;
    this.models = [];
    this.listeners = new Set();

    this.loadFromStorage();
  }

  /**
   * Load session from local storage
   */
  loadFromStorage() {
    try {
      const stored = localStorage.getItem(SESSION_STORAGE_KEY);
      if (stored) {
        const data = JSON.parse(stored);

        // Check if session is still valid
        if (data.expiresAt && new Date(data.expiresAt) > new Date()) {
          this.sessionId = data.sessionId;
          this.expiresAt = data.expiresAt;
          this.models = data.models || [];
        } else {
          // Session expired, clear it
          this.clearStorage();
        }
      }
    } catch (e) {
      console.error('Failed to load session from storage:', e);
      this.clearStorage();
    }
  }

  /**
   * Save session to local storage
   */
  saveToStorage() {
    try {
      const data = {
        sessionId: this.sessionId,
        expiresAt: this.expiresAt,
        models: this.models,
      };
      localStorage.setItem(SESSION_STORAGE_KEY, JSON.stringify(data));
    } catch (e) {
      console.error('Failed to save session to storage:', e);
    }
  }

  /**
   * Clear session from storage
   */
  clearStorage() {
    localStorage.removeItem(SESSION_STORAGE_KEY);
    this.sessionId = null;
    this.expiresAt = null;
    this.models = [];
  }

  /**
   * Initialize or get existing session
   * @returns {Promise<string>} Session ID
   */
  async ensureSession() {
    if (this.sessionId && !this.isExpired()) {
      return this.sessionId;
    }

    // Create new session
    const response = await createSession();
    this.sessionId = response.session_id;
    this.expiresAt = response.expires_at;
    this.models = [];

    this.saveToStorage();
    this.notifyListeners('session-created', { sessionId: this.sessionId });

    return this.sessionId;
  }

  /**
   * Check if current session is expired
   * @returns {boolean}
   */
  isExpired() {
    if (!this.expiresAt) {
      return true;
    }
    return new Date(this.expiresAt) <= new Date();
  }

  /**
   * Get current session ID
   * @returns {string|null}
   */
  getSessionId() {
    if (this.isExpired()) {
      this.clearStorage();
      return null;
    }
    return this.sessionId;
  }

  /**
   * Set session ID directly (for SSR hydration)
   * @param {string} sessionId - Session ID from server
   * @param {string} [expiresAt] - Optional expiration timestamp
   */
  setSessionId(sessionId, expiresAt = null) {
    this.sessionId = sessionId;

    // If no expiration provided, set to 24 hours from now
    if (expiresAt) {
      this.expiresAt = expiresAt;
    } else {
      const expires = new Date();
      expires.setHours(expires.getHours() + 24);
      this.expiresAt = expires.toISOString();
    }

    this.models = [];
    this.saveToStorage();
    this.notifyListeners('session-created', { sessionId: this.sessionId });
  }

  /**
   * Add a model to the session
   * @param {Object} modelData - Model information
   */
  addModel(modelData) {
    this.models.push(modelData);
    this.saveToStorage();
    this.notifyListeners('model-added', modelData);
  }

  /**
   * Remove a model from the session
   * @param {string} modelId - Model ID to remove
   */
  removeModel(modelId) {
    const index = this.models.findIndex(m => m.model_id === modelId);
    if (index !== -1) {
      const removed = this.models.splice(index, 1)[0];
      this.saveToStorage();
      this.notifyListeners('model-removed', removed);
    }
  }

  /**
   * Update model configuration
   * @param {string} modelId - Model ID
   * @param {Object} updates - Properties to update
   */
  updateModel(modelId, updates) {
    const model = this.models.find(m => m.model_id === modelId);
    if (model) {
      Object.assign(model, updates);
      this.saveToStorage();
      this.notifyListeners('model-updated', model);
    }
  }

  /**
   * Get all models in session
   * @returns {Array}
   */
  getModels() {
    return [...this.models];
  }

  /**
   * Get a specific model by ID
   * @param {string} modelId
   * @returns {Object|null}
   */
  getModel(modelId) {
    return this.models.find(m => m.model_id === modelId) || null;
  }

  /**
   * Check if all models have materials assigned
   * @returns {boolean}
   */
  allModelsConfigured() {
    return this.models.length > 0 && this.models.every(m => m.material_id);
  }

  /**
   * Add event listener
   * @param {function} callback - Listener callback
   */
  addListener(callback) {
    this.listeners.add(callback);
  }

  /**
   * Remove event listener
   * @param {function} callback - Listener to remove
   */
  removeListener(callback) {
    this.listeners.delete(callback);
  }

  /**
   * Notify all listeners of an event
   * @param {string} eventType - Event type
   * @param {any} data - Event data
   */
  notifyListeners(eventType, data) {
    for (const listener of this.listeners) {
      try {
        listener(eventType, data);
      } catch (e) {
        console.error('Listener error:', e);
      }
    }
  }

  /**
   * Clear current session
   */
  clearSession() {
    this.clearStorage();
    this.notifyListeners('session-cleared', null);
  }
}

// Export singleton instance
export const sessionManager = new SessionManager();
export default sessionManager;
