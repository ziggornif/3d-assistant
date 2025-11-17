/**
 * Admin Panel - Main Entry Point
 */

// State
let materials = [];

// DOM Elements (only elements used in admin panel, not login page)
const addMaterialForm = document.getElementById('add-material-form');
const materialsListDiv = document.getElementById('materials-list');
const pricingHistoryDiv = document.getElementById('pricing-history');
const notificationContainer = document.getElementById('notification-container');
const editModal = document.getElementById('edit-modal');
const editMaterialForm = document.getElementById('edit-material-form');

// Initialize
document.addEventListener('DOMContentLoaded', () => {
  setupEventListeners();
  // Load data if authenticated (SSR provides initial data)
  if (window.__SSR_DATA__ && window.__SSR_DATA__.authenticated) {
    materials = window.__SSR_DATA__.materials || [];
  }
});

function setupEventListeners() {
  // Login form submits naturally to /admin/login (no JS interception needed)
  // Only attach listeners if elements exist (they won't exist on login page)
  if (addMaterialForm) addMaterialForm.addEventListener('submit', handleAddMaterial);
  if (editMaterialForm) editMaterialForm.addEventListener('submit', handleEditMaterialSubmit);

  // Attach event listeners to SSR-rendered buttons (using event delegation)
  if (materialsListDiv) {
    materialsListDiv.addEventListener('click', handleMaterialsListClick);
  }

  // Close modal when clicking outside
  if (editModal) {
    editModal.addEventListener('click', e => {
      if (e.target === editModal) {
        closeEditModal();
      }
    });
  }

  // Close modal on Escape key
  document.addEventListener('keydown', e => {
    if (e.key === 'Escape' && editModal && !editModal.hidden) {
      closeEditModal();
    }
  });
}

// Event delegation handler for materials list buttons
function handleMaterialsListClick(e) {
  const target = e.target;

  // Handle edit button click
  if (target.classList.contains('edit-btn')) {
    const id = target.dataset.id;
    if (id) {
      openEditModal(id);
    }
  }

  // Handle toggle button click
  if (target.classList.contains('toggle-btn')) {
    const id = target.dataset.id;
    const isActive = target.dataset.active === 'true';
    if (id) {
      window.toggleMaterial(id, !isActive);
    }
  }
}

// Notification System
function showNotification(message, type = 'info', duration = 4000) {
  const notification = document.createElement('div');
  notification.className = `notification notification-${type}`;
  notification.innerHTML = `
    <span>${escapeHtml(message)}</span>
    <button class="notification-close" onclick="this.parentElement.remove()">&times;</button>
  `;

  notificationContainer.appendChild(notification);

  // Auto-remove after duration
  setTimeout(() => {
    notification.classList.add('hiding');
    setTimeout(() => notification.remove(), 300);
  }, duration);
}

// Modal Functions
function openEditModal(materialId) {
  const material = materials.find(m => m.id === materialId);
  if (!material) {
    return;
  }

  document.getElementById('edit-material-id').value = material.id;
  document.getElementById('edit-name').value = material.name;
  document.getElementById('edit-price').value = material.price_per_cm3;
  document.getElementById('edit-description').value = material.description || '';
  document.getElementById('edit-color').value = material.color || '#CCCCCC';

  editModal.hidden = false;
}

function closeEditModal() {
  editModal.hidden = true;
  editMaterialForm.reset();
}

// Expose to window for inline onclick handlers
window.closeEditModal = closeEditModal;

async function handleEditMaterialSubmit(e) {
  e.preventDefault();

  const id = document.getElementById('edit-material-id').value;
  const updates = {
    name: document.getElementById('edit-name').value,
    price_per_cm3: parseFloat(document.getElementById('edit-price').value),
    description: document.getElementById('edit-description').value || null,
    color: document.getElementById('edit-color').value,
  };

  if (isNaN(updates.price_per_cm3) || updates.price_per_cm3 < 0) {
    showNotification('Prix invalide', 'error');
    return;
  }

  try {
    const response = await fetch(`/api/admin/materials/${id}`, {
      method: 'PATCH',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(updates),
    });

    if (!response.ok) {
      throw new Error('Failed to update');
    }

    closeEditModal();
    await loadMaterials();
    await loadPricingHistory();
    showNotification('Matériau mis à jour avec succès', 'success');
  } catch (_err) {
    showNotification('Erreur lors de la mise à jour', 'error');
  }
}

// Authentication is now handled server-side via form POST to /admin/login
// Logout is handled via form POST to /admin/logout

async function loadMaterials() {
  try {
    const response = await fetch('/api/admin/materials');

    if (!response.ok) {
      throw new Error('Failed to load materials');
    }

    materials = await response.json();
    renderMaterials();
  } catch (_err) {
    if (materialsListDiv) {
      materialsListDiv.innerHTML = '<p class="error-message">Erreur de chargement</p>';
    }
  }
}

function renderMaterials() {
  if (materials.length === 0) {
    materialsListDiv.innerHTML = '<p>Aucun matériau</p>';
    return;
  }

  const html = `
    <table>
      <thead>
        <tr>
          <th>Couleur</th>
          <th>Nom</th>
          <th>Prix/cm³</th>
          <th>Statut</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        ${materials
          .map(
            m => `
          <tr>
            <td><span class="color-preview" style="background-color: ${m.color || '#CCCCCC'}"></span></td>
            <td>${escapeHtml(m.name)}</td>
            <td>${m.price_per_cm3.toFixed(3)} €</td>
            <td><span class="status-badge ${m.active ? 'status-active' : 'status-inactive'}">${m.active ? 'Actif' : 'Inactif'}</span></td>
            <td>
              <button class="action-btn btn-edit" onclick="editMaterial('${m.id}')">Modifier</button>
              <button class="action-btn btn-toggle" onclick="toggleMaterial('${m.id}', ${!m.active})">${m.active ? 'Désactiver' : 'Activer'}</button>
            </td>
          </tr>
        `
          )
          .join('')}
      </tbody>
    </table>
  `;
  materialsListDiv.innerHTML = html;
}

async function handleAddMaterial(e) {
  e.preventDefault();

  const material = {
    name: document.getElementById('new-name').value,
    price_per_cm3: parseFloat(document.getElementById('new-price').value),
    description: document.getElementById('new-description').value || null,
    color: document.getElementById('new-color').value,
    service_type_id: document.getElementById('new-service-type').value,
    properties: null,
  };

  try {
    const response = await fetch('/api/admin/materials', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(material),
    });

    if (!response.ok) {
      throw new Error('Failed to create material');
    }

    addMaterialForm.reset();
    document.getElementById('new-color').value = '#CCCCCC';
    await loadMaterials();
    await loadPricingHistory();
    showNotification('Matériau ajouté avec succès', 'success');
  } catch (_err) {
    showNotification('Erreur lors de la création du matériau', 'error');
  }
}

// Global function for onclick handlers
window.editMaterial = function (id) {
  openEditModal(id);
};

window.toggleMaterial = async function (id, active) {
  try {
    const response = await fetch(`/api/admin/materials/${id}`, {
      method: 'PATCH',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ active }),
    });

    if (!response.ok) {
      throw new Error('Failed to toggle');
    }

    await loadMaterials();
    showNotification(active ? 'Matériau activé' : 'Matériau désactivé', 'success');
  } catch (_err) {
    showNotification('Erreur lors du changement de statut', 'error');
  }
};

async function loadPricingHistory() {
  try {
    const response = await fetch('/api/admin/pricing-history');

    if (!response.ok) {
      throw new Error('Failed to load history');
    }

    const history = await response.json();
    renderPricingHistory(history);
  } catch (_err) {
    if (pricingHistoryDiv) {
      pricingHistoryDiv.innerHTML = '<p class="error-message">Erreur de chargement</p>';
    }
  }
}

function renderPricingHistory(history) {
  if (history.length === 0) {
    pricingHistoryDiv.innerHTML = '<p>Aucun historique</p>';
    return;
  }

  const html = `
    <div class="history-list">
      ${history
        .map(
          entry => `
        <div class="history-entry">
          <div>
            <span class="history-material">${escapeHtml(entry.material_name)}</span>
            <span class="history-change">
              ${entry.old_price ? `${entry.old_price.toFixed(3)} € → ` : 'Création: '}${entry.new_price.toFixed(3)} €
            </span>
          </div>
          <div class="history-date">${new Date(entry.changed_at).toLocaleString('fr-FR')}</div>
        </div>
      `
        )
        .join('')}
    </div>
  `;
  pricingHistoryDiv.innerHTML = html;
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}
