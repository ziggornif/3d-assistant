/**
 * File Uploader Web Component
 * Handles drag-and-drop and click-to-upload for STL/3MF files
 */

import { uploadModel } from '../services/api-client.js';
import { sessionManager } from '../services/session-manager.js';

class FileUploader extends HTMLElement {
  constructor() {
    super();
    this.attachShadow({ mode: 'open' });
    this.maxSizeMb = 50;
    this.acceptedFormats = ['stl', '3mf'];
  }

  connectedCallback() {
    this.maxSizeMb = parseInt(this.getAttribute('max-size-mb') || '50', 10);
    const formats = this.getAttribute('accepted-formats');
    if (formats) {
      this.acceptedFormats = formats.split(',').map(f => f.trim().toLowerCase());
    }

    this.render();
    this.setupEventListeners();
  }

  render() {
    this.shadowRoot.innerHTML = `
      <style>
        :host {
          display: block;
        }

        .drop-zone {
          border: 3px dashed var(--color-border, #e2e8f0);
          border-radius: var(--border-radius-lg, 0.75rem);
          padding: 2rem;
          text-align: center;
          transition: all 0.3s ease;
          background: var(--color-surface, #ffffff);
          cursor: pointer;
        }

        .drop-zone:hover,
        .drop-zone.dragover {
          border-color: var(--color-primary, #2563eb);
          background: rgba(37, 99, 235, 0.05);
        }

        .drop-zone:focus-within {
          outline: 3px solid var(--color-primary, #2563eb);
          outline-offset: 2px;
        }

        .drop-icon {
          font-size: 3rem;
          margin-bottom: 1rem;
        }

        .drop-text {
          font-size: 1.125rem;
          font-weight: 500;
          margin-bottom: 0.5rem;
        }

        .drop-hint {
          color: var(--color-text-light, #64748b);
          font-size: 0.875rem;
        }

        input[type="file"] {
          position: absolute;
          width: 1px;
          height: 1px;
          padding: 0;
          margin: -1px;
          overflow: hidden;
          clip: rect(0, 0, 0, 0);
          border: 0;
        }

        .progress-container {
          margin-top: 1rem;
          display: none;
        }

        .progress-container.active {
          display: block;
        }

        .progress-bar {
          width: 100%;
          height: 8px;
          background: var(--color-border, #e2e8f0);
          border-radius: 4px;
          overflow: hidden;
        }

        .progress-fill {
          height: 100%;
          background: var(--color-primary, #2563eb);
          transition: width 0.3s ease;
          width: 0%;
        }

        .progress-text {
          margin-top: 0.5rem;
          font-size: 0.875rem;
          color: var(--color-text-light, #64748b);
        }

        .error-message {
          color: var(--color-error, #dc2626);
          margin-top: 1rem;
          padding: 0.75rem;
          background: rgba(220, 38, 38, 0.1);
          border-radius: 0.5rem;
          display: none;
        }

        .error-message.visible {
          display: block;
        }

        .btn-browse {
          display: inline-block;
          background: var(--color-primary, #2563eb);
          color: white;
          border: none;
          padding: 0.75rem 1.5rem;
          border-radius: 0.5rem;
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
          font-size: 1rem;
          font-weight: 500;
          cursor: pointer;
          margin-top: 1rem;
          min-height: 44px;
          min-width: 44px;
          text-align: center;
        }

        .btn-browse:hover {
          background: var(--color-primary-dark, #1d4ed8);
        }

        .btn-browse:focus-visible {
          outline: 3px solid var(--color-primary, #2563eb);
          outline-offset: 2px;
        }
      </style>

      <div class="drop-zone">
        <div class="drop-icon" aria-hidden="true">📁</div>
        <div class="drop-text" id="drop-text">Glissez vos fichiers ici</div>
        <div class="drop-hint">
          ou cliquez pour parcourir<br>
          Formats: ${this.acceptedFormats.map(f => f.toUpperCase()).join(', ')} | Max: ${this.maxSizeMb} MB
        </div>
        <button class="btn-browse" type="button" aria-describedby="drop-text">
          Parcourir les fichiers
        </button>
        <input
          type="file"
          id="file-input"
          accept="${this.acceptedFormats.map(f => '.' + f).join(',')}"
          multiple
          aria-label="Sélectionner des fichiers à télécharger"
        >
      </div>

      <div class="progress-container" role="progressbar" aria-valuenow="0" aria-valuemin="0" aria-valuemax="100">
        <div class="progress-bar">
          <div class="progress-fill"></div>
        </div>
        <div class="progress-text">Téléchargement en cours...</div>
      </div>

      <div class="error-message" role="alert" aria-live="assertive"></div>
    `;
  }

  setupEventListeners() {
    const dropZone = this.shadowRoot.querySelector('.drop-zone');
    const fileInput = this.shadowRoot.querySelector('#file-input');
    const browseBtn = this.shadowRoot.querySelector('.btn-browse');

    // Drag and drop events
    ['dragenter', 'dragover', 'dragleave', 'drop'].forEach(eventName => {
      dropZone.addEventListener(eventName, e => {
        e.preventDefault();
        e.stopPropagation();
      });
    });

    ['dragenter', 'dragover'].forEach(eventName => {
      dropZone.addEventListener(eventName, () => {
        dropZone.classList.add('dragover');
      });
    });

    ['dragleave', 'drop'].forEach(eventName => {
      dropZone.addEventListener(eventName, () => {
        dropZone.classList.remove('dragover');
      });
    });

    dropZone.addEventListener('drop', e => {
      const files = e.dataTransfer.files;
      this.handleFiles(files);
    });

    // Click to browse button
    browseBtn.addEventListener('click', e => {
      e.stopPropagation();
      fileInput.click();
    });

    // Click on drop zone area
    dropZone.addEventListener('click', e => {
      if (e.target !== browseBtn) {
        fileInput.click();
      }
    });

    // File input change
    fileInput.addEventListener('change', e => {
      this.handleFiles(e.target.files);
      fileInput.value = ''; // Reset for same file selection
    });
  }

  async handleFiles(fileList) {
    const files = Array.from(fileList);

    if (files.length === 0) {
      return;
    }

    // Validate files
    const validFiles = [];
    for (const file of files) {
      const validation = this.validateFile(file);
      if (!validation.valid) {
        this.showError(validation.error);
        return;
      }
      validFiles.push(file);
    }

    // Upload each file
    for (const file of validFiles) {
      await this.uploadFile(file);
    }
  }

  validateFile(file) {
    // Check size
    const sizeMb = file.size / (1024 * 1024);
    if (sizeMb > this.maxSizeMb) {
      return {
        valid: false,
        error: `Le fichier "${file.name}" est trop volumineux (${sizeMb.toFixed(1)} MB). Maximum: ${this.maxSizeMb} MB`,
      };
    }

    // Check format
    const ext = file.name.split('.').pop().toLowerCase();
    if (!this.acceptedFormats.includes(ext)) {
      return {
        valid: false,
        error: `Format non supporté: .${ext}. Formats acceptés: ${this.acceptedFormats.join(', ')}`,
      };
    }

    return { valid: true };
  }

  async uploadFile(file) {
    this.hideError();
    this.showProgress();

    try {
      // Ensure we have a session
      const sessionId = await sessionManager.getSessionId();

      // Upload with progress tracking
      const result = await uploadModel(sessionId, file, progress => {
        this.updateProgress(progress);
      });

      this.hideProgress();

      // Dispatch success event
      this.dispatchEvent(
        new CustomEvent('upload-complete', {
          detail: result,
          bubbles: true,
          composed: true,
        })
      );
    } catch (error) {
      this.hideProgress();
      this.showError(error.message || 'Erreur lors du téléchargement');

      this.dispatchEvent(
        new CustomEvent('upload-error', {
          detail: { error, file },
          bubbles: true,
          composed: true,
        })
      );
    }
  }

  showProgress() {
    const container = this.shadowRoot.querySelector('.progress-container');
    container.classList.add('active');
    this.updateProgress(0);
  }

  updateProgress(percent) {
    const fill = this.shadowRoot.querySelector('.progress-fill');
    const text = this.shadowRoot.querySelector('.progress-text');
    const container = this.shadowRoot.querySelector('.progress-container');

    fill.style.width = `${percent}%`;
    text.textContent = `Téléchargement: ${Math.round(percent)}%`;
    container.setAttribute('aria-valuenow', Math.round(percent));
  }

  hideProgress() {
    const container = this.shadowRoot.querySelector('.progress-container');
    container.classList.remove('active');
  }

  showError(message) {
    const errorEl = this.shadowRoot.querySelector('.error-message');
    errorEl.textContent = message;
    errorEl.classList.add('visible');
  }

  hideError() {
    const errorEl = this.shadowRoot.querySelector('.error-message');
    errorEl.classList.remove('visible');
  }
}

// Register the custom element
customElements.define('file-uploader', FileUploader);

export default FileUploader;
