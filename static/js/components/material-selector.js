/**
 * Material Selector Web Component
 * Displays available materials and allows selection with price estimation
 */

class MaterialSelector extends HTMLElement {
  constructor() {
    super();
    this.attachShadow({ mode: 'open' });
    this.materials = [];
    this.selectedMaterialId = null;
    this.modelId = null;
    this.volumeCm3 = 0;
  }

  static get observedAttributes() {
    return ['model-id', 'volume-cm3', 'selected-material'];
  }

  connectedCallback() {
    this.render();
    this.loadMaterials();
  }

  attributeChangedCallback(name, oldValue, newValue) {
    if (oldValue === newValue) {
      return;
    }

    if (name === 'model-id') {
      this.modelId = newValue;
    }
    if (name === 'volume-cm3') {
      this.volumeCm3 = parseFloat(newValue) || 0;
      this.updatePriceEstimates();
    }
    if (name === 'selected-material') {
      this.selectedMaterialId = newValue;
      this.updateSelection();
    }
  }

  render() {
    this.shadowRoot.innerHTML = `
      <style>
        :host {
          display: block;
          margin-top: 1rem;
        }

        .material-selector {
          background: white;
          border: 1px solid var(--color-border, #e2e8f0);
          border-radius: var(--border-radius-md, 0.5rem);
          padding: 1rem;
        }

        .selector-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 1rem;
        }

        .selector-title {
          font-weight: 600;
          color: var(--color-text, #1e293b);
          margin: 0;
        }

        .estimated-price {
          font-size: 1.25rem;
          font-weight: 700;
          color: var(--color-primary, #2563eb);
        }

        .materials-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
          gap: 0.75rem;
        }

        .material-option {
          border: 2px solid var(--color-border, #e2e8f0);
          border-radius: var(--border-radius-sm, 0.375rem);
          padding: 0.75rem;
          cursor: pointer;
          transition: all 0.2s ease;
          background: white;
        }

        .material-option:hover {
          border-color: var(--color-primary, #2563eb);
          box-shadow: 0 2px 8px rgba(37, 99, 235, 0.1);
        }

        .material-option.selected {
          border-color: var(--color-primary, #2563eb);
          background: rgba(37, 99, 235, 0.05);
        }

        .material-option:focus {
          outline: 2px solid var(--color-primary, #2563eb);
          outline-offset: 2px;
        }

        .material-header {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          margin-bottom: 0.5rem;
        }

        .color-swatch {
          width: 20px;
          height: 20px;
          border-radius: 50%;
          border: 1px solid rgba(0, 0, 0, 0.2);
          flex-shrink: 0;
        }

        .material-name {
          font-weight: 600;
          font-size: 0.875rem;
          color: var(--color-text, #1e293b);
        }

        .material-description {
          font-size: 0.75rem;
          color: var(--color-text-light, #64748b);
          margin-bottom: 0.5rem;
          line-height: 1.4;
        }

        .material-price {
          font-size: 0.875rem;
          font-weight: 600;
          color: var(--color-success, #16a34a);
        }

        .material-properties {
          display: flex;
          gap: 0.25rem;
          margin-top: 0.5rem;
          flex-wrap: wrap;
        }

        .property-badge {
          font-size: 0.625rem;
          padding: 0.125rem 0.375rem;
          background: var(--color-bg-light, #f1f5f9);
          border-radius: 9999px;
          color: var(--color-text-light, #64748b);
        }

        .loading {
          text-align: center;
          padding: 2rem;
          color: var(--color-text-light, #64748b);
        }

        .error {
          background: rgba(220, 38, 38, 0.1);
          color: var(--color-error, #dc2626);
          padding: 1rem;
          border-radius: var(--border-radius-sm, 0.375rem);
          text-align: center;
        }

        .check-icon {
          position: absolute;
          top: 0.5rem;
          right: 0.5rem;
          color: var(--color-primary, #2563eb);
          font-size: 1.25rem;
        }

        .material-option {
          position: relative;
        }

        .selection-indicator {
          display: none;
          position: absolute;
          top: 0.5rem;
          right: 0.5rem;
          width: 24px;
          height: 24px;
          background: var(--color-primary, #2563eb);
          border-radius: 50%;
          color: white;
          font-size: 0.875rem;
          line-height: 24px;
          text-align: center;
        }

        .material-option.selected .selection-indicator {
          display: block;
        }
      </style>

      <div class="material-selector">
        <div class="selector-header">
          <h3 class="selector-title">Choix du matériau</h3>
          <div class="estimated-price" id="total-estimate" aria-live="polite">-</div>
        </div>
        <div class="materials-grid" id="materials-container" role="radiogroup" aria-label="Sélection du matériau">
          <div class="loading">Chargement des matériaux...</div>
        </div>
      </div>
    `;
  }

  async loadMaterials() {
    try {
      // Check for SSR cached materials first
      if (window.__MATERIALS_CACHE__ && window.__MATERIALS_CACHE__.length > 0) {
        console.info('Using SSR cached materials');
        this.materials = window.__MATERIALS_CACHE__;
      } else {
        // Fallback to API fetch
        const response = await fetch('http://127.0.0.1:3000/api/materials');
        if (!response.ok) {
          throw new Error('Failed to load materials');
        }
        this.materials = await response.json();
      }

      this.renderMaterials();

      this.dispatchEvent(
        new CustomEvent('materials-loaded', {
          detail: { materials: this.materials },
          bubbles: true,
          composed: true,
        })
      );
    } catch (error) {
      console.error('Error loading materials:', error);
      this.showError('Erreur lors du chargement des matériaux');
    }
  }

  renderMaterials() {
    const container = this.shadowRoot.getElementById('materials-container');

    if (this.materials.length === 0) {
      container.innerHTML = '<div class="error">Aucun matériau disponible</div>';
      return;
    }

    container.innerHTML = this.materials
      .map(
        material => `
        <div class="material-option ${this.selectedMaterialId === material.id ? 'selected' : ''}"
             data-material-id="${material.id}"
             role="radio"
             aria-checked="${this.selectedMaterialId === material.id}"
             tabindex="0"
             aria-label="${material.name} - ${this.formatPrice(this.calculatePrice(material))}">
          <div class="selection-indicator" aria-hidden="true">✓</div>
          <div class="material-header">
            <div class="color-swatch" style="background-color: ${material.color || '#CCCCCC'}"></div>
            <span class="material-name">${material.name}</span>
          </div>
          <div class="material-description">${material.description || ''}</div>
          <div class="material-price" data-price="${material.price_per_cm3}">
            ${this.formatPrice(this.calculatePrice(material))}
          </div>
          ${this.renderProperties(material.properties)}
        </div>
      `
      )
      .join('');

    // Add event listeners
    container.querySelectorAll('.material-option').forEach(option => {
      option.addEventListener('click', () => this.selectMaterial(option.dataset.materialId));
      option.addEventListener('keydown', e => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          this.selectMaterial(option.dataset.materialId);
        }
      });
    });
  }

  renderProperties(properties) {
    if (!properties) {
      return '';
    }

    const badges = [];
    if (properties.strength) {
      badges.push(`Force: ${this.translateStrength(properties.strength)}`);
    }
    if (properties.flexibility) {
      badges.push(`Flex: ${this.translateFlexibility(properties.flexibility)}`);
    }

    if (badges.length === 0) {
      return '';
    }

    return `
      <div class="material-properties">
        ${badges.map(badge => `<span class="property-badge">${badge}</span>`).join('')}
      </div>
    `;
  }

  translateStrength(value) {
    const map = {
      low: 'Faible',
      medium: 'Moyenne',
      high: 'Haute',
      very_high: 'Très haute',
    };
    return map[value] || value;
  }

  translateFlexibility(value) {
    const map = {
      low: 'Rigide',
      medium: 'Moyenne',
      high: 'Flexible',
      very_high: 'Très flexible',
    };
    return map[value] || value;
  }

  calculatePrice(material) {
    return material.price_per_cm3 * this.volumeCm3;
  }

  formatPrice(amount) {
    return new Intl.NumberFormat('fr-FR', {
      style: 'currency',
      currency: 'EUR',
    }).format(amount);
  }

  updatePriceEstimates() {
    if (!this.shadowRoot) {
      return;
    }

    this.shadowRoot.querySelectorAll('.material-option').forEach(option => {
      const materialId = option.dataset.materialId;
      const material = this.materials.find(m => m.id === materialId);
      if (material) {
        const priceElement = option.querySelector('.material-price');
        if (priceElement) {
          priceElement.textContent = this.formatPrice(this.calculatePrice(material));
        }
      }
    });

    this.updateTotalEstimate();
  }

  updateTotalEstimate() {
    if (!this.shadowRoot) {
      return;
    }

    const totalElement = this.shadowRoot.getElementById('total-estimate');
    if (!totalElement) {
      return;
    }

    if (!this.selectedMaterialId) {
      totalElement.textContent = '-';
      return;
    }

    const material = this.materials.find(m => m.id === this.selectedMaterialId);
    if (material) {
      totalElement.textContent = this.formatPrice(this.calculatePrice(material));
    }
  }

  updateSelection() {
    if (!this.shadowRoot) {
      return;
    }

    this.shadowRoot.querySelectorAll('.material-option').forEach(option => {
      const isSelected = option.dataset.materialId === this.selectedMaterialId;
      option.classList.toggle('selected', isSelected);
      option.setAttribute('aria-checked', isSelected);
    });

    this.updateTotalEstimate();
  }

  selectMaterial(materialId) {
    this.selectedMaterialId = materialId;
    this.updateSelection();

    const material = this.materials.find(m => m.id === materialId);
    const estimatedPrice = material ? this.calculatePrice(material) : 0;

    this.dispatchEvent(
      new CustomEvent('material-selected', {
        detail: {
          materialId,
          modelId: this.modelId,
          estimatedPrice,
          material,
        },
        bubbles: true,
        composed: true,
      })
    );
  }

  showError(message) {
    const container = this.shadowRoot.getElementById('materials-container');
    container.innerHTML = `<div class="error">${message}</div>`;
  }

  // Public method to get current selection
  getSelection() {
    return {
      materialId: this.selectedMaterialId,
      material: this.materials.find(m => m.id === this.selectedMaterialId),
      estimatedPrice: this.selectedMaterialId
        ? this.calculatePrice(this.materials.find(m => m.id === this.selectedMaterialId))
        : 0,
    };
  }
}

customElements.define('material-selector', MaterialSelector);

export default MaterialSelector;
