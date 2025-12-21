/**
 * Session Manager for managing user quote sessions
 */

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
    this.listeners = new Set();
    if (window.__SSR_DATA__ && window.__SSR_DATA__.sessionId) {
      this.sessionId = window.__SSR_DATA__.sessionId;
    }
    this.models = [];
  }

  /**
   * Get current session ID
   * @returns {string|null}
   */
  getSessionId() {
    return this.sessionId;
  }

  /**
   * Set session ID directly (for SSR hydration)
   * @param {string} sessionId - Session ID from server
   * @param {string} [expiresAt] - Optional expiration timestamp
   */
  setSessionId(sessionId) {
    this.sessionId = sessionId;
    this.models = [];
    this.notifyListeners('session-created', { sessionId: this.sessionId });
  }

  /**
   * Add a model to the session
   * @param {Object} modelData - Model information
   */
  addModel(modelData) {
    this.models.push(modelData);
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
    this.sessionId = null;
    this.models = [];
    this.notifyListeners('session-cleared', null);
  }
}

// Export singleton instance
export const sessionManager = new SessionManager();
export default sessionManager;
