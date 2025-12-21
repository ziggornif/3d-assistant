/**
 * Main application entry point
 * Bootstraps the 3D Quote Service frontend
 */

import { sessionManager } from './services/session-manager.js';
import { checkHealth } from './services/api-client.js';
import { initAccessibility } from './utils/accessibility.js';

// Import Web Components
import './components/file-uploader.js';
import './components/model-viewer.js';
import './components/material-selector.js';
import './components/quote-summary.js';

// Application state
const app = {
  initialized: false,
  apiAvailable: false,
};

/**
 * Initialize the application
 */
async function init() {
  console.info('Initializing 3D Quote Service...');

  // Initialize accessibility features
  initAccessibility();

  // Check for SSR data (Server-Side Rendered)
  if (window.__SSR_DATA__) {
    console.info('SSR data detected, hydrating from server-rendered content');

    // Use pre-created session from server
    if (window.__SSR_DATA__.sessionId) {
      sessionManager.setSessionId(window.__SSR_DATA__.sessionId);
      console.info('Session hydrated from SSR:', window.__SSR_DATA__.sessionId);
    }

    // Store materials for components to use
    if (window.__SSR_DATA__.materials) {
      window.__MATERIALS_CACHE__ = window.__SSR_DATA__.materials;
      console.info('Materials cached from SSR:', window.__SSR_DATA__.materials.length);
    }

    app.apiAvailable = true;
  } else {
    // Fallback to API-based initialization (for development or old pages)
    console.info('No SSR data, falling back to API initialization');

    // Check API availability
    app.apiAvailable = await checkHealth();

    if (!app.apiAvailable) {
      showError('Le service est temporairement indisponible. Veuillez réessayer plus tard.');
      console.error('API health check failed');
    }

    // Load or create session
    try {
      await sessionManager.ensureSession();
      console.info('Session initialized:', sessionManager.getSessionId());
    } catch (e) {
      console.error('Failed to initialize session:', e);
      showError('Impossible de créer une session. Veuillez réessayer.');
    }
  }

  // Setup event listeners
  setupEventListeners();

  // Restore any existing models
  restoreModels();

  app.initialized = true;
  console.info('Application initialized successfully');
}

/**
 * Setup global event listeners
 */
function setupEventListeners() {
  // Listen for session changes
  sessionManager.addListener((eventType, data) => {
    switch (eventType) {
      case 'model-added':
        onModelAdded(data);
        break;
      case 'model-removed':
        onModelRemoved(data);
        break;
      case 'model-updated':
        onModelUpdated(data);
        break;
      case 'session-cleared':
        onSessionCleared();
        break;
    }
  });

  // Listen for file-uploader events
  const fileUploader = document.getElementById('file-uploader');
  if (fileUploader) {
    fileUploader.addEventListener('upload-complete', e => {
      const modelData = e.detail;
      sessionManager.addModel(modelData);
    });

    fileUploader.addEventListener('upload-error', e => {
      showError(e.detail.message || 'Erreur lors du téléchargement');
    });
  }

  // Listen for quote updates
  const quoteSummary = document.getElementById('quote-summary');
  if (quoteSummary) {
    quoteSummary.addEventListener('quote-requested', () => {
      updateQuote();
    });
  }
}

/**
 * Restore models from session storage
 */

import { getSessionModels } from './services/api-client.js';

async function restoreModels() {
  const sessionId = sessionManager.getSessionId();
  if (!sessionId) return;

  try {
    const models = await getSessionModels(sessionId);
    sessionManager.models = [];
    if (Array.isArray(models)) {
      models.forEach(rawModel => {
        const model = {
          ...rawModel,
          model_id: rawModel.id,
          dimensions_mm:
            typeof rawModel.dimensions_mm === 'string'
              ? JSON.parse(rawModel.dimensions_mm)
              : rawModel.dimensions_mm,
          support_analysis:
            typeof rawModel.support_analysis === 'string'
              ? JSON.parse(rawModel.support_analysis)
              : rawModel.support_analysis,
        };
        sessionManager.addModel(model);
      });
    }
    if (models.length > 0) {
      const modelsSection = document.getElementById('models-section');
      if (modelsSection) {
        modelsSection.hidden = false;
      }
      sessionManager.getModels().forEach(model => {
        renderModel(model);
      });
      updateQuoteVisibility();
    }
  } catch (e) {
    console.error('Erreur lors du chargement des modèles de la session:', e);
  }
}

/**
 * Handle new model added
 * @param {Object} modelData
 */
function onModelAdded(modelData) {
  const modelsSection = document.getElementById('models-section');
  if (modelsSection) {
    modelsSection.hidden = false;
  }

  renderModel(modelData);
  updateQuoteVisibility();
}

/**
 * Handle model removed
 * @param {Object} modelData
 */
function onModelRemoved(modelData) {
  const modelCard = document.getElementById(`model-${modelData.model_id}`);
  if (modelCard) {
    modelCard.remove();
  }

  updateQuoteVisibility();

  // Hide models section if no models left
  if (sessionManager.getModels().length === 0) {
    const modelsSection = document.getElementById('models-section');
    if (modelsSection) {
      modelsSection.hidden = true;
    }
  }
}

/**
 * Handle model configuration updated
 * @param {Object} modelData
 */
function onModelUpdated(modelData) {
  // Update the model card display
  const priceElement = document.querySelector(`#model-${modelData.model_id} .estimated-price`);
  if (priceElement && modelData.estimated_price) {
    priceElement.textContent = `${modelData.estimated_price.toFixed(2)} €`;
  }

  updateQuoteVisibility();
}

/**
 * Handle session cleared
 */
function onSessionCleared() {
  const modelsList = document.getElementById('models-list');
  if (modelsList) {
    modelsList.innerHTML = '';
  }

  const modelsSection = document.getElementById('models-section');
  if (modelsSection) {
    modelsSection.hidden = true;
  }

  const quoteSection = document.getElementById('quote-section');
  if (quoteSection) {
    quoteSection.hidden = true;
  }
}

/**
 * Render a model card
 * @param {Object} modelData
 */
function renderModel(modelData) {
  const modelsList = document.getElementById('models-list');
  if (!modelsList) {
    return;
  }

  const card = document.createElement('div');
  card.id = `model-${modelData.model_id}`;
  card.className = 'model-card';
  card.setAttribute('role', 'listitem');

  // Get material color if available
  let materialColor = '';
  if (modelData.material_id && window.__MATERIALS_CACHE__) {
    const material = window.__MATERIALS_CACHE__.find(m => m.id === modelData.material_id);
    if (material && material.color) {
      materialColor = `model-color="${material.color}"`;
    }
  }

  card.innerHTML = `
    <div class="model-info">
      <h3>${escapeHtml(modelData.filename)}</h3>
      <p>Volume: ${modelData.volume_cm3.toFixed(2)} cm³</p>
      <p>Dimensions: ${modelData.dimensions_mm.x.toFixed(1)} × ${modelData.dimensions_mm.y.toFixed(1)} × ${modelData.dimensions_mm.z.toFixed(1)} mm</p>
    </div>
    <div class="model-viewer-container">
      <model-viewer model-url="${modelData.preview_url}" auto-rotate="true" ${materialColor}></model-viewer>
    </div>
    <div class="model-config">
      <material-selector
        model-id="${modelData.model_id}"
        volume-cm3="${modelData.volume_cm3}"
        ${modelData.material_id ? `selected-material="${modelData.material_id}"` : ''}
      ></material-selector>
    </div>
    <button class="btn-secondary remove-model" aria-label="Supprimer ${escapeHtml(modelData.filename)}">
      Supprimer
    </button>
  `;

  // Add remove handler
  const removeBtn = card.querySelector('.remove-model');
  removeBtn.addEventListener('click', () => {
    removeModel(modelData.model_id);
  });

  // Add material selection handler
  const materialSelector = card.querySelector('material-selector');
  const modelViewer = card.querySelector('model-viewer');

  materialSelector.addEventListener('material-selected', async e => {
    const { materialId, modelId, estimatedPrice, material } = e.detail;

    // Update 3D viewer color if material has a color
    if (material && material.color && modelViewer) {
      modelViewer.setAttribute('model-color', material.color);
    }

    await onMaterialSelected(modelId, materialId, estimatedPrice);
  });

  modelsList.appendChild(card);
}

/**
 * Remove a model
 * @param {string} modelId
 */
async function removeModel(modelId) {
  try {
    const { deleteModel } = await import('./services/api-client.js');
    await deleteModel(sessionManager.getSessionId(), modelId);
    sessionManager.removeModel(modelId);
  } catch (e) {
    console.error('Failed to remove model:', e);
    showError('Impossible de supprimer le modèle');
  }
}

/**
 * Handle material selection for a model
 * @param {string} modelId
 * @param {string} materialId
 * @param {number} estimatedPrice
 */
async function onMaterialSelected(modelId, materialId, _estimatedPrice) {
  try {
    const { configureModel } = await import('./services/api-client.js');
    const result = await configureModel(sessionManager.getSessionId(), modelId, materialId);

    // Update local session storage
    sessionManager.updateModel(modelId, {
      material_id: materialId,
      estimated_price: result.estimated_price,
    });

    console.info(
      `Model ${modelId} configured with material ${materialId}, price: ${result.estimated_price}€`
    );

    // Refresh quote summary
    refreshQuoteSummary();
  } catch (e) {
    console.error('Failed to configure model:', e);
    showError('Impossible de configurer le matériau');
  }
}

/**
 * Update quote section visibility
 */
function updateQuoteVisibility() {
  const quoteSection = document.getElementById('quote-section');
  const models = sessionManager.getModels();

  if (quoteSection) {
    const hasConfiguredModels = models.some(m => m.material_id);
    quoteSection.hidden = !hasConfiguredModels;

    // Set session ID on quote summary if visible
    if (hasConfiguredModels) {
      const quoteSummary = document.getElementById('quote-summary');
      if (quoteSummary) {
        quoteSummary.setAttribute('session-id', sessionManager.getSessionId());
      }
    }
  }
}

/**
 * Refresh quote summary component
 */
function refreshQuoteSummary() {
  const quoteSummary = document.getElementById('quote-summary');
  if (quoteSummary) {
    quoteSummary.setAttribute('session-id', sessionManager.getSessionId());
    quoteSummary.refresh();
  }
  updateQuoteVisibility();
}

/**
 * Update quote calculation
 */
async function updateQuote() {
  try {
    const { getCurrentQuote } = await import('./services/api-client.js');
    const quote = await getCurrentQuote(sessionManager.getSessionId());

    const quoteSummary = document.getElementById('quote-summary');
    if (quoteSummary) {
      quoteSummary.setQuoteData(quote);
    }
  } catch (e) {
    console.error('Failed to update quote:', e);
  }
}

/**
 * Show error message to user
 * @param {string} message
 */
function showError(message) {
  // Create or update error banner
  let errorBanner = document.getElementById('error-banner');

  if (!errorBanner) {
    errorBanner = document.createElement('div');
    errorBanner.id = 'error-banner';
    errorBanner.className = 'error-banner';
    errorBanner.setAttribute('role', 'alert');
    errorBanner.setAttribute('aria-live', 'assertive');

    const main = document.querySelector('main');
    if (main) {
      main.insertBefore(errorBanner, main.firstChild);
    }
  }

  errorBanner.textContent = message;
  errorBanner.hidden = false;

  // Auto-hide after 10 seconds
  setTimeout(() => {
    errorBanner.hidden = true;
  }, 10000);
}

/**
 * Escape HTML to prevent XSS
 * @param {string} text
 * @returns {string}
 */
function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}

// Export for testing
export { app, init };
