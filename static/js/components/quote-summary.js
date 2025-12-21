/**
 * Quote Summary Web Component
 * Displays itemized quote breakdown with totals
 */

class QuoteSummary extends HTMLElement {
  constructor() {
    super();
    this.attachShadow({ mode: 'open' });
    this.quoteData = null;
    this.loading = false;
  }

  static get observedAttributes() {
    return ['session-id'];
  }

  connectedCallback() {
    this.render();
  }

  attributeChangedCallback(name, oldValue, newValue) {
    if (oldValue === newValue) {
      return;
    }

    if (name === 'session-id' && newValue) {
      this.loadQuote(newValue);
    }
  }

  render() {
    this.shadowRoot.innerHTML = `
      <style>
        :host {
          display: block;
        }

        .quote-summary {
          background: white;
          border: 1px solid var(--color-border, #e2e8f0);
          border-radius: var(--border-radius-lg, 0.75rem);
          overflow: hidden;
        }

        .summary-header {
          background: var(--color-primary, #2563eb);
          color: white;
          padding: 1rem;
          text-align: center;
        }

        .summary-header h3 {
          margin: 0;
          font-size: 1.25rem;
        }

        .items-list {
          padding: 0;
          margin: 0;
          list-style: none;
        }

        .quote-item {
          display: flex;
          justify-content: space-between;
          padding: 1rem;
          border-bottom: 1px solid var(--color-border, #e2e8f0);
        }

        .quote-item:last-child {
          border-bottom: none;
        }

        .item-details {
          flex: 1;
        }

        .item-name {
          font-weight: 600;
          color: var(--color-text, #1e293b);
          margin-bottom: 0.25rem;
        }

        .item-meta {
          font-size: 0.75rem;
          color: var(--color-text-light, #64748b);
        }

        .item-price {
          font-weight: 600;
          color: var(--color-success, #16a34a);
          white-space: nowrap;
        }

        .totals {
          background: var(--color-bg-light, #f8fafc);
          padding: 1rem;
        }

        .total-row {
          display: flex;
          justify-content: space-between;
          margin-bottom: 0.5rem;
        }

        .total-row:last-child {
          margin-bottom: 0;
        }

        .total-label {
          color: var(--color-text-light, #64748b);
        }

        .total-value {
          font-weight: 600;
        }

        .grand-total {
          border-top: 2px solid var(--color-border, #e2e8f0);
          padding-top: 0.5rem;
          margin-top: 0.5rem;
        }

        .grand-total .total-label,
        .grand-total .total-value {
          font-size: 1.25rem;
          color: var(--color-primary, #2563eb);
          font-weight: 700;
        }

        .actions {
          padding: 1rem;
          text-align: center;
        }

        .btn-generate {
          background: var(--color-primary, #2563eb);
          color: white;
          border: none;
          padding: 0.75rem 2rem;
          border-radius: var(--border-radius-md, 0.5rem);
          font-size: 1rem;
          font-weight: 600;
          cursor: pointer;
          transition: background 0.2s;
        }

        .btn-generate:hover {
          background: var(--color-primary-dark, #1d4ed8);
        }

        .btn-generate:disabled {
          background: var(--color-secondary, #64748b);
          cursor: not-allowed;
        }

        .loading {
          text-align: center;
          padding: 2rem;
          color: var(--color-text-light, #64748b);
        }

        .empty-state {
          text-align: center;
          padding: 2rem;
          color: var(--color-text-light, #64748b);
        }

        .error {
          background: rgba(220, 38, 38, 0.1);
          color: var(--color-error, #dc2626);
          padding: 1rem;
          text-align: center;
        }

        .quote-id {
          font-size: 0.75rem;
          color: var(--color-text-light, #64748b);
          text-align: center;
          padding: 0.5rem;
          background: var(--color-bg-light, #f8fafc);
        }

        .minimum-notice {
          background: rgba(245, 158, 11, 0.1);
          border: 1px solid rgba(245, 158, 11, 0.3);
          border-radius: var(--border-radius-sm, 0.375rem);
          padding: 0.75rem;
          margin-top: 0.5rem;
          font-size: 0.875rem;
          color: #92400e;
        }

        .minimum-notice strong {
          display: block;
          margin-bottom: 0.25rem;
        }

        .original-total {
          text-decoration: line-through;
          color: var(--color-text-light, #64748b);
          font-size: 0.875rem;
        }
      </style>

      <div class="quote-summary">
        <div class="summary-header">
          <h3>Récapitulatif du devis</h3>
        </div>
        <div id="content">
          <div class="empty-state">
            Sélectionnez des matériaux pour voir le devis
          </div>
        </div>
      </div>
    `;
  }

  async loadQuote(sessionId) {
    if (this.loading) {
      return;
    }

    this.loading = true;
    this.showLoading();

    try {
      const response = await fetch(`http://127.0.0.1:3000/api/sessions/${sessionId}/quote`);

      if (!response.ok) {
        throw new Error('Failed to load quote');
      }

      this.quoteData = await response.json();
      this.renderQuote();

      this.dispatchEvent(
        new CustomEvent('quote-loaded', {
          detail: this.quoteData,
          bubbles: true,
          composed: true,
        })
      );
    } catch (error) {
      console.error('Error loading quote:', error);
      this.showError('Erreur lors du chargement du devis');
    } finally {
      this.loading = false;
    }
  }

  showLoading() {
    const content = this.shadowRoot.getElementById('content');
    content.innerHTML = '<div class="loading">Calcul du devis en cours...</div>';
  }

  showError(message) {
    const content = this.shadowRoot.getElementById('content');
    content.innerHTML = `<div class="error">${message}</div>`;
  }

  renderQuote() {
    if (!this.quoteData || this.quoteData.items.length === 0) {
      this.showEmpty();
      return;
    }

    const content = this.shadowRoot.getElementById('content');

    const itemsHtml = this.quoteData.items
      .map(
        item => `
        <li class="quote-item">
          <div class="item-details">
            <div class="item-name">${this.escapeHtml(item.model_name)}</div>
            <div class="item-meta">
              ${this.escapeHtml(item.material_name)} - ${item.volume_cm3.toFixed(2)} cm³
            </div>
          </div>
          <div class="item-price">${this.formatPrice(item.price)}</div>
        </li>
      `
      )
      .join('');

    const minimumNoticeHtml = this.quoteData.minimum_applied
      ? `
        <div class="minimum-notice">
          <strong>Minimum de commande appliqué</strong>
          Le total calculé (${this.formatPrice(this.quoteData.calculated_total)}) est inférieur au minimum de commande de ${this.formatPrice(this.quoteData.total)}.
        </div>
      `
      : '';

    content.innerHTML = `
      <ul class="items-list" role="list">
        ${itemsHtml}
      </ul>
      <div class="totals">
        <div class="total-row">
          <span class="total-label">Sous-total matériaux</span>
          <span class="total-value">${this.formatPrice(this.quoteData.subtotal)}</span>
        </div>
        <div class="total-row">
          <span class="total-label">Frais de service</span>
          <span class="total-value">${this.formatPrice(this.quoteData.fees)}</span>
        </div>
        <div class="total-row grand-total">
          <span class="total-label">Total</span>
          <span class="total-value">
            ${this.quoteData.minimum_applied ? `<span class="original-total">${this.formatPrice(this.quoteData.calculated_total)}</span> ` : ''}
            ${this.formatPrice(this.quoteData.total)}
          </span>
        </div>
        ${minimumNoticeHtml}
      </div>
      ${
        this.quoteData.quote_id
          ? `<div class="quote-id">Devis #${this.quoteData.quote_id.substring(0, 8)}</div>`
          : ''
      }
      <div class="actions">
        <button class="btn-generate" id="generate-btn">
          ${this.quoteData.quote_id ? 'Regénérer le devis' : 'Finaliser le devis'}
        </button>
      </div>
    `;

    // Add generate button handler
    const generateBtn = this.shadowRoot.getElementById('generate-btn');
    generateBtn.addEventListener('click', () => this.generateQuote());
  }

  showEmpty() {
    const content = this.shadowRoot.getElementById('content');
    content.innerHTML = `
      <div class="empty-state">
        Sélectionnez des matériaux pour voir le devis
      </div>
    `;
  }

  async generateQuote() {
    const sessionId = this.getAttribute('session-id');
    if (!sessionId) {
      return;
    }

    const generateBtn = this.shadowRoot.getElementById('generate-btn');
    if (generateBtn) {
      generateBtn.disabled = true;
      generateBtn.textContent = 'Génération...';
    }

    try {
      const response = await fetch(`http://127.0.0.1:3000/api/sessions/${sessionId}/quote`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error('Failed to generate quote');
      }

      this.quoteData = await response.json();
      this.renderQuote();

      this.dispatchEvent(
        new CustomEvent('quote-generated', {
          detail: this.quoteData,
          bubbles: true,
          composed: true,
        })
      );
    } catch (error) {
      console.error('Error generating quote:', error);
      this.showError('Erreur lors de la génération du devis');
    }
  }

  formatPrice(amount) {
    return new Intl.NumberFormat('fr-FR', {
      style: 'currency',
      currency: 'EUR',
    }).format(amount);
  }

  escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  // Public method to refresh quote
  refresh() {
    const sessionId = this.getAttribute('session-id');
    if (sessionId) {
      this.loadQuote(sessionId);
    }
  }

  // Public method to set quote data directly
  setQuoteData(data) {
    this.quoteData = data;
    this.renderQuote();
  }
}

customElements.define('quote-summary', QuoteSummary);

export default QuoteSummary;
