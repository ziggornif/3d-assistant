/**
 * Model Viewer Web Component
 * Renders 3D models using Three.js with interactive controls
 */

import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { STLLoader } from 'three/addons/loaders/STLLoader.js';
import { ThreeMFLoader } from 'three/addons/loaders/3MFLoader.js';

class ModelViewer extends HTMLElement {
  constructor() {
    super();
    this.attachShadow({ mode: 'open' });
    this.scene = null;
    this.camera = null;
    this.renderer = null;
    this.controls = null;
    this.mesh = null;
    this.animationId = null;
    this.gridHelper = null;
    this.mainLight = null;
    this.groundPlane = null;
    this.fallbackMode = false;
  }

  static get observedAttributes() {
    return ['model-url', 'auto-rotate', 'show-dimensions'];
  }

  connectedCallback() {
    this.render();
    this.initScene();
  }

  disconnectedCallback() {
    this.cleanup();
  }

  attributeChangedCallback(name, oldValue, newValue) {
    if (oldValue === newValue) {
      return;
    }

    if (name === 'model-url' && this.scene) {
      this.loadModel(newValue);
    }
    if (name === 'auto-rotate' && this.controls) {
      this.controls.autoRotate = newValue === 'true';
    }
  }

  render() {
    this.shadowRoot.innerHTML = `
      <style>
        :host {
          display: block;
          position: relative;
          width: 100%;
          height: 300px;
          min-height: 200px;
        }

        .viewer-container {
          width: 100%;
          height: 100%;
          background: #f0f0f0;
          border-radius: var(--border-radius-md, 0.5rem);
          overflow: hidden;
          position: relative;
        }

        canvas {
          width: 100%;
          height: 100%;
          display: block;
        }

        .loading-message,
        .error-message {
          position: absolute;
          top: 50%;
          left: 50%;
          transform: translate(-50%, -50%);
          text-align: center;
          padding: 1rem;
        }

        .loading-message {
          color: var(--color-text-light, #64748b);
        }

        .error-message {
          color: var(--color-error, #dc2626);
          background: rgba(220, 38, 38, 0.1);
          border-radius: 0.5rem;
        }

        .dimensions-overlay {
          position: absolute;
          bottom: 10px;
          left: 10px;
          background: rgba(0, 0, 0, 0.7);
          color: white;
          padding: 0.5rem;
          border-radius: 0.25rem;
          font-size: 0.75rem;
          font-family: monospace;
        }

        .controls-hint {
          position: absolute;
          top: 10px;
          right: 10px;
          background: rgba(0, 0, 0, 0.5);
          color: white;
          padding: 0.25rem 0.5rem;
          border-radius: 0.25rem;
          font-size: 0.625rem;
          opacity: 0.8;
        }

        .webgl-error {
          background: rgba(220, 38, 38, 0.1);
          padding: 2rem;
          text-align: center;
          border-radius: var(--border-radius-md, 0.5rem);
        }

        .fallback-container {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          height: 100%;
          background: linear-gradient(135deg, #f0f0f0 0%, #e0e0e0 100%);
          padding: 2rem;
        }

        .fallback-icon {
          font-size: 3rem;
          margin-bottom: 1rem;
          opacity: 0.6;
        }

        .fallback-title {
          font-size: 1.125rem;
          font-weight: 600;
          color: #374151;
          margin-bottom: 0.5rem;
        }

        .fallback-message {
          color: #6b7280;
          font-size: 0.875rem;
          margin-bottom: 1rem;
          text-align: center;
        }

        .fallback-dimensions {
          background: white;
          padding: 1rem;
          border-radius: 0.5rem;
          box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
          font-family: monospace;
          font-size: 0.875rem;
        }

        .fallback-dimensions dt {
          font-weight: 600;
          color: #374151;
          display: inline;
        }

        .fallback-dimensions dd {
          display: inline;
          margin-left: 0.5rem;
          color: #2563eb;
        }
      </style>

      <div class="viewer-container" role="img" aria-label="Visualisation 3D du modèle">
        <div class="loading-message">Initialisation...</div>
      </div>
    `;
  }

  async waitForThreeJS() {
    return new Promise(resolve => {
      const check = () => {
        if (typeof THREE !== 'undefined') {
          resolve();
        } else {
          setTimeout(check, 100);
        }
      };
      check();
    });
  }

  initScene() {
    // Check WebGL support
    if (!this.checkWebGLSupport()) {
      this.showFallback();
      return;
    }

    const container = this.shadowRoot.querySelector('.viewer-container');
    container.innerHTML = '';

    // Create scene
    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color(0xf0f0f0);

    // Create camera
    const width = container.clientWidth;
    const height = container.clientHeight;
    this.camera = new THREE.PerspectiveCamera(45, width / height, 0.1, 10000);
    this.camera.position.set(100, 100, 100);

    // Create renderer with shadow support
    this.renderer = new THREE.WebGLRenderer({ antialias: true });
    this.renderer.setSize(width, height);
    this.renderer.setPixelRatio(window.devicePixelRatio);
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    container.appendChild(this.renderer.domElement);

    // Add lights - high contrast setup to show relief
    const ambientLight = new THREE.AmbientLight(0xffffff, 0.3);
    this.scene.add(ambientLight);

    const directionalLight1 = new THREE.DirectionalLight(0xffffff, 1.5);
    directionalLight1.position.set(100, 200, 150);
    directionalLight1.castShadow = true;
    directionalLight1.shadow.mapSize.width = 2048;
    directionalLight1.shadow.mapSize.height = 2048;
    directionalLight1.shadow.camera.near = 0.5;
    directionalLight1.shadow.camera.far = 1000;
    directionalLight1.shadow.camera.left = -200;
    directionalLight1.shadow.camera.right = 200;
    directionalLight1.shadow.camera.top = 200;
    directionalLight1.shadow.camera.bottom = -200;
    directionalLight1.shadow.bias = -0.0001;
    this.scene.add(directionalLight1);
    this.mainLight = directionalLight1;

    const directionalLight2 = new THREE.DirectionalLight(0xffffff, 0.4);
    directionalLight2.position.set(-100, 150, -100);
    this.scene.add(directionalLight2);

    // Add ground plane for shadows
    const groundGeometry = new THREE.PlaneGeometry(2000, 2000);
    const groundMaterial = new THREE.ShadowMaterial({ opacity: 0.3 });
    this.groundPlane = new THREE.Mesh(groundGeometry, groundMaterial);
    this.groundPlane.rotation.x = -Math.PI / 2;
    this.groundPlane.position.y = 0;
    this.groundPlane.receiveShadow = true;
    this.scene.add(this.groundPlane);

    // Add grid helper (will be resized based on model)
    this.gridHelper = new THREE.GridHelper(200, 20);
    this.scene.add(this.gridHelper);

    // Setup controls (OrbitControls)
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.enableDamping = true;
    this.controls.dampingFactor = 0.05;
    this.controls.autoRotate = this.getAttribute('auto-rotate') === 'true';
    this.controls.autoRotateSpeed = 2;

    // Add controls hint
    const hint = document.createElement('div');
    hint.className = 'controls-hint';
    hint.textContent = 'Clic + glisser: Rotation | Molette: Zoom | Clic droit: Pan';
    container.appendChild(hint);

    // Handle resize
    this.resizeObserver = new ResizeObserver(() => this.onResize());
    this.resizeObserver.observe(container);

    // Start animation loop
    this.animate();

    // Load model if URL provided
    const modelUrl = this.getAttribute('model-url');
    if (modelUrl) {
      this.loadModel(modelUrl);
    }

    this.dispatchEvent(
      new CustomEvent('viewer-ready', {
        bubbles: true,
        composed: true,
      })
    );
  }

  loadModel(url) {
    if (!this.scene) {
      return;
    }

    // Remove existing mesh
    if (this.mesh) {
      this.scene.remove(this.mesh);
      if (this.mesh.geometry) {
        this.mesh.geometry.dispose();
      }
      if (this.mesh.material) {
        if (Array.isArray(this.mesh.material)) {
          this.mesh.material.forEach(m => m.dispose());
        } else {
          this.mesh.material.dispose();
        }
      }
      this.mesh = null;
    }

    this.showMessage('Chargement du modèle...');

    const fullUrl = `http://127.0.0.1:3000${url}`;

    // Determine file format from URL
    const is3MF = url.toLowerCase().endsWith('.3mf');

    if (is3MF) {
      this.load3MFModel(fullUrl, url);
    } else {
      this.loadSTLModel(fullUrl, url);
    }
  }

  loadSTLModel(fullUrl, url) {
    const loader = new STLLoader();

    loader.load(
      fullUrl,
      geometry => {
        this.processLoadedGeometry(geometry, url);
      },
      progress => {
        const percent = (progress.loaded / progress.total) * 100;
        this.showMessage(`Chargement: ${Math.round(percent)}%`);
      },
      error => {
        console.error('Error loading STL:', error);
        this.showError('Erreur lors du chargement du modèle STL');
      }
    );
  }

  load3MFModel(fullUrl, url) {
    const loader = new ThreeMFLoader();

    loader.load(
      fullUrl,
      object3D => {
        // 3MF loader returns a Group, we need to extract geometry
        let geometry = null;

        object3D.traverse(child => {
          if (child.isMesh && !geometry) {
            geometry = child.geometry.clone();
          }
        });

        if (geometry) {
          this.processLoadedGeometry(geometry, url);
        } else {
          this.showError('Fichier 3MF vide ou invalide');
        }
      },
      progress => {
        if (progress.total > 0) {
          const percent = (progress.loaded / progress.total) * 100;
          this.showMessage(`Chargement: ${Math.round(percent)}%`);
        }
      },
      error => {
        console.error('Error loading 3MF:', error);
        this.showError('Erreur lors du chargement du modèle 3MF');
      }
    );
  }

  processLoadedGeometry(geometry, url) {
    // Rotate from Z-up (slicer convention) to Y-up (Three.js convention)
    geometry.rotateX(-Math.PI / 2);

    // Center geometry at origin
    geometry.center();

    // Compute bounding box to position base on grid
    geometry.computeBoundingBox();
    const boundingBox = geometry.boundingBox;
    const yOffset = -boundingBox.min.y; // Move up so bottom touches Y=0

    // Apply Y translation to geometry
    geometry.translate(0, yOffset, 0);

    const material = new THREE.MeshPhongMaterial({
      color: 0xffaa00, // Orange clair style slicer
      specular: 0x222222,
      shininess: 30,
      flatShading: true, // Show triangle facets for better relief visibility
    });

    this.mesh = new THREE.Mesh(geometry, material);
    this.mesh.castShadow = true;
    this.mesh.receiveShadow = true;
    this.scene.add(this.mesh);

    // Update grid size based on model
    this.updateGridSize();

    // Update shadow camera to fit model
    this.updateShadowCamera();

    // Center camera on model
    this.fitCameraToModel();

    // Show dimensions if requested
    if (this.getAttribute('show-dimensions') === 'true') {
      const box = new THREE.Box3().setFromObject(this.mesh);
      const size = box.getSize(new THREE.Vector3());
      this.showDimensions({ x: size.x, y: size.y, z: size.z });
    }

    this.hideMessage();

    this.dispatchEvent(
      new CustomEvent('model-loaded', {
        detail: { url },
        bubbles: true,
        composed: true,
      })
    );
  }

  updateGridSize() {
    if (!this.mesh || !this.gridHelper) {
      return;
    }

    const box = new THREE.Box3().setFromObject(this.mesh);
    const size = box.getSize(new THREE.Vector3());

    // Calculate grid size based on model footprint (X and Z dimensions)
    const maxFootprint = Math.max(size.x, size.z);
    // Round up to nearest 50 and add padding
    const gridSize = Math.ceil(maxFootprint / 50) * 50 * 2;
    const divisions = Math.max(10, Math.floor(gridSize / 10));

    // Remove old grid and create new one
    this.scene.remove(this.gridHelper);
    this.gridHelper = new THREE.GridHelper(gridSize, divisions);
    this.scene.add(this.gridHelper);
  }

  updateShadowCamera() {
    if (!this.mesh || !this.mainLight) {
      return;
    }

    const box = new THREE.Box3().setFromObject(this.mesh);
    const size = box.getSize(new THREE.Vector3());
    const center = box.getCenter(new THREE.Vector3());

    // Adjust shadow camera to encompass the model
    const maxDim = Math.max(size.x, size.y, size.z) * 1.5;

    this.mainLight.shadow.camera.left = -maxDim;
    this.mainLight.shadow.camera.right = maxDim;
    this.mainLight.shadow.camera.top = maxDim;
    this.mainLight.shadow.camera.bottom = -maxDim;
    this.mainLight.shadow.camera.far = maxDim * 4;

    // Position light relative to model size
    this.mainLight.position.set(
      center.x + maxDim,
      center.y + maxDim * 2,
      center.z + maxDim
    );
    this.mainLight.target.position.copy(center);
    this.scene.add(this.mainLight.target);

    this.mainLight.shadow.camera.updateProjectionMatrix();
  }

  fitCameraToModel() {
    if (!this.mesh) {
      return;
    }

    const box = new THREE.Box3().setFromObject(this.mesh);
    const center = box.getCenter(new THREE.Vector3());
    const size = box.getSize(new THREE.Vector3());

    const maxDim = Math.max(size.x, size.y, size.z);
    const fov = this.camera.fov * (Math.PI / 180);
    let cameraZ = Math.abs(maxDim / 2 / Math.tan(fov / 2));
    cameraZ *= 2; // Add some padding

    // Position camera to look at model from an angle
    this.camera.position.set(cameraZ * 0.8, cameraZ * 0.6, cameraZ * 0.8);
    this.camera.lookAt(center);

    if (this.controls) {
      this.controls.target.copy(center);
      this.controls.update();
    }
  }

  showDimensions(dims) {
    const container = this.shadowRoot.querySelector('.viewer-container');
    let overlay = container.querySelector('.dimensions-overlay');

    if (!overlay) {
      overlay = document.createElement('div');
      overlay.className = 'dimensions-overlay';
      container.appendChild(overlay);
    }

    overlay.innerHTML = `
      X: ${dims.x.toFixed(1)} mm<br>
      Y: ${dims.y.toFixed(1)} mm<br>
      Z: ${dims.z.toFixed(1)} mm
    `;
  }

  animate() {
    this.animationId = requestAnimationFrame(() => this.animate());

    if (this.controls) {
      this.controls.update();
    }

    if (this.renderer && this.scene && this.camera) {
      this.renderer.render(this.scene, this.camera);
    }
  }

  onResize() {
    if (!this.renderer || !this.camera) {
      return;
    }

    const container = this.shadowRoot.querySelector('.viewer-container');
    const width = container.clientWidth;
    const height = container.clientHeight;

    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(width, height);
  }

  checkWebGLSupport() {
    try {
      const canvas = document.createElement('canvas');
      return !!(
        window.WebGLRenderingContext &&
        (canvas.getContext('webgl') || canvas.getContext('experimental-webgl'))
      );
    } catch (_e) {
      return false;
    }
  }

  showMessage(text) {
    const container = this.shadowRoot.querySelector('.viewer-container');
    let msg = container.querySelector('.loading-message');
    if (!msg) {
      msg = document.createElement('div');
      msg.className = 'loading-message';
      container.appendChild(msg);
    }
    msg.textContent = text;
    msg.style.display = 'block';
  }

  hideMessage() {
    const msg = this.shadowRoot.querySelector('.loading-message');
    if (msg) {
      msg.style.display = 'none';
    }
  }

  showError(text) {
    const container = this.shadowRoot.querySelector('.viewer-container');
    container.innerHTML = `<div class="error-message">${text}</div>`;
  }

  showFallback() {
    const container = this.shadowRoot.querySelector('.viewer-container');
    this.fallbackMode = true;

    container.innerHTML = `
      <div class="fallback-container">
        <div class="fallback-icon">📦</div>
        <div class="fallback-title">Prévisualisation 3D non disponible</div>
        <div class="fallback-message">
          Votre navigateur ne supporte pas WebGL.<br>
          Les dimensions du modèle seront affichées après le chargement.
        </div>
        <div id="fallback-info" class="fallback-dimensions" style="display: none;">
          <dl>
            <dt>Largeur (X):</dt><dd id="dim-x">--</dd><br>
            <dt>Hauteur (Y):</dt><dd id="dim-y">--</dd><br>
            <dt>Profondeur (Z):</dt><dd id="dim-z">--</dd>
          </dl>
        </div>
      </div>
    `;

    // Listen for dimension updates from parent
    this.dispatchEvent(
      new CustomEvent('viewer-ready', {
        detail: { fallbackMode: true },
        bubbles: true,
        composed: true,
      })
    );
  }

  updateFallbackDimensions(dims) {
    if (!this.fallbackMode) {
      return;
    }

    const info = this.shadowRoot.getElementById('fallback-info');
    const dimX = this.shadowRoot.getElementById('dim-x');
    const dimY = this.shadowRoot.getElementById('dim-y');
    const dimZ = this.shadowRoot.getElementById('dim-z');

    if (info && dimX && dimY && dimZ) {
      dimX.textContent = `${dims.x.toFixed(1)} mm`;
      dimY.textContent = `${dims.y.toFixed(1)} mm`;
      dimZ.textContent = `${dims.z.toFixed(1)} mm`;
      info.style.display = 'block';
    }
  }

  cleanup() {
    if (this.animationId) {
      cancelAnimationFrame(this.animationId);
    }

    if (this.resizeObserver) {
      this.resizeObserver.disconnect();
    }

    if (this.mesh) {
      this.mesh.geometry.dispose();
      this.mesh.material.dispose();
    }

    if (this.renderer) {
      this.renderer.dispose();
    }
  }
}

// Register the custom element
customElements.define('model-viewer', ModelViewer);

export default ModelViewer;
