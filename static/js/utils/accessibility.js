/**
 * Accessibility Utilities for RGAA Compliance
 * WCAG 2.1 AA compliant keyboard navigation and screen reader support
 */

/**
 * Trap focus within a modal or dialog (WCAG 2.4.3)
 * @param {HTMLElement} container - The container to trap focus within
 * @returns {Function} Cleanup function to remove event listener
 */
export function trapFocus(container) {
  const focusableElements = container.querySelectorAll(
    'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
  );

  if (focusableElements.length === 0) {
    return () => {};
  }

  const firstElement = focusableElements[0];
  const lastElement = focusableElements[focusableElements.length - 1];

  const handleKeyDown = event => {
    if (event.key !== 'Tab') {
      return;
    }

    if (event.shiftKey) {
      // Shift + Tab
      if (document.activeElement === firstElement) {
        event.preventDefault();
        lastElement.focus();
      }
    } else {
      // Tab
      if (document.activeElement === lastElement) {
        event.preventDefault();
        firstElement.focus();
      }
    }
  };

  container.addEventListener('keydown', handleKeyDown);
  firstElement.focus();

  return () => {
    container.removeEventListener('keydown', handleKeyDown);
  };
}

/**
 * Announce message to screen readers (WCAG 4.1.3)
 * @param {string} message - Message to announce
 * @param {'polite'|'assertive'} priority - Announcement priority
 */
export function announce(message, priority = 'polite') {
  const announcer = document.getElementById('sr-announcer') || createAnnouncer();
  announcer.setAttribute('aria-live', priority);
  announcer.textContent = '';

  // Small delay to ensure screen reader picks up the change
  setTimeout(() => {
    announcer.textContent = message;
  }, 100);
}

/**
 * Create a screen reader announcer element
 * @returns {HTMLElement} The announcer element
 */
function createAnnouncer() {
  const announcer = document.createElement('div');
  announcer.id = 'sr-announcer';
  announcer.className = 'sr-only';
  announcer.setAttribute('role', 'status');
  announcer.setAttribute('aria-live', 'polite');
  announcer.setAttribute('aria-atomic', 'true');
  document.body.appendChild(announcer);
  return announcer;
}

/**
 * Enable arrow key navigation for a group of items (WCAG 2.1.1)
 * @param {HTMLElement} container - Container with navigable items
 * @param {string} itemSelector - CSS selector for items
 * @param {'horizontal'|'vertical'|'both'} direction - Navigation direction
 */
export function enableArrowNavigation(container, itemSelector, direction = 'vertical') {
  const getItems = () => container.querySelectorAll(itemSelector);

  container.addEventListener('keydown', event => {
    const items = getItems();
    if (items.length === 0) {
      return;
    }

    const currentIndex = Array.from(items).indexOf(document.activeElement);
    if (currentIndex === -1) {
      return;
    }

    let newIndex = currentIndex;

    if ((direction === 'vertical' || direction === 'both') && event.key === 'ArrowDown') {
      event.preventDefault();
      newIndex = (currentIndex + 1) % items.length;
    } else if ((direction === 'vertical' || direction === 'both') && event.key === 'ArrowUp') {
      event.preventDefault();
      newIndex = (currentIndex - 1 + items.length) % items.length;
    } else if ((direction === 'horizontal' || direction === 'both') && event.key === 'ArrowRight') {
      event.preventDefault();
      newIndex = (currentIndex + 1) % items.length;
    } else if ((direction === 'horizontal' || direction === 'both') && event.key === 'ArrowLeft') {
      event.preventDefault();
      newIndex = (currentIndex - 1 + items.length) % items.length;
    } else if (event.key === 'Home') {
      event.preventDefault();
      newIndex = 0;
    } else if (event.key === 'End') {
      event.preventDefault();
      newIndex = items.length - 1;
    }

    if (newIndex !== currentIndex) {
      items[newIndex].focus();
    }
  });
}

/**
 * Setup roving tabindex for a widget (WCAG 2.4.3)
 * @param {HTMLElement} container - Container element
 * @param {string} itemSelector - CSS selector for items
 */
export function setupRovingTabindex(container, itemSelector) {
  const items = container.querySelectorAll(itemSelector);
  if (items.length === 0) {
    return;
  }

  // Set initial tabindex
  items.forEach((item, index) => {
    item.setAttribute('tabindex', index === 0 ? '0' : '-1');
  });

  container.addEventListener('focusin', event => {
    const target = event.target;
    if (target.matches(itemSelector)) {
      items.forEach(item => item.setAttribute('tabindex', '-1'));
      target.setAttribute('tabindex', '0');
    }
  });
}

/**
 * Check if user prefers reduced motion
 * @returns {boolean} True if user prefers reduced motion
 */
export function prefersReducedMotion() {
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

/**
 * Validate form field and show error (WCAG 3.3.1)
 * @param {HTMLInputElement} field - Input field to validate
 * @param {string} errorMessage - Error message to display
 * @returns {boolean} True if valid, false otherwise
 */
export function validateField(field, errorMessage) {
  const errorId = `${field.id}-error`;
  let errorElement = document.getElementById(errorId);

  if (!field.checkValidity() || (field.required && !field.value.trim())) {
    // Show error
    field.classList.add('field-error');
    field.setAttribute('aria-invalid', 'true');
    field.setAttribute('aria-describedby', errorId);

    if (!errorElement) {
      errorElement = document.createElement('div');
      errorElement.id = errorId;
      errorElement.className = 'field-error-message';
      errorElement.setAttribute('role', 'alert');
      field.parentNode.appendChild(errorElement);
    }

    errorElement.textContent = errorMessage;
    announce(errorMessage, 'assertive');
    return false;
  } else {
    // Clear error
    field.classList.remove('field-error');
    field.removeAttribute('aria-invalid');
    field.removeAttribute('aria-describedby');

    if (errorElement) {
      errorElement.remove();
    }
    return true;
  }
}

/**
 * Create accessible button with proper ARIA attributes
 * @param {Object} options - Button options
 * @returns {HTMLButtonElement} Button element
 */
export function createAccessibleButton({
  text,
  ariaLabel,
  onClick,
  className = 'btn-primary',
  disabled = false,
}) {
  const button = document.createElement('button');
  button.textContent = text;
  button.className = className;
  button.type = 'button';

  if (ariaLabel) {
    button.setAttribute('aria-label', ariaLabel);
  }

  if (disabled) {
    button.disabled = true;
    button.setAttribute('aria-disabled', 'true');
  }

  button.addEventListener('click', onClick);
  return button;
}

/**
 * Setup escape key to close modal (WCAG 2.1.1)
 * @param {HTMLElement} modal - Modal element
 * @param {Function} closeFunction - Function to call on escape
 */
export function setupEscapeClose(modal, closeFunction) {
  const handler = event => {
    if (event.key === 'Escape' && !modal.hidden) {
      closeFunction();
    }
  };

  document.addEventListener('keydown', handler);
  return () => document.removeEventListener('keydown', handler);
}

/**
 * Check contrast ratio between two colors
 * @param {string} color1 - First color in hex
 * @param {string} color2 - Second color in hex
 * @returns {number} Contrast ratio
 */
export function checkContrastRatio(color1, color2) {
  const luminance = hex => {
    const rgb = parseInt(hex.slice(1), 16);
    const r = ((rgb >> 16) & 255) / 255;
    const g = ((rgb >> 8) & 255) / 255;
    const b = (rgb & 255) / 255;

    const [rs, gs, bs] = [r, g, b].map(c =>
      c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4)
    );

    return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs;
  };

  const l1 = luminance(color1);
  const l2 = luminance(color2);

  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);

  return (lighter + 0.05) / (darker + 0.05);
}

/**
 * Initialize accessibility features on page load
 */
export function initAccessibility() {
  // Create announcer
  createAnnouncer();

  // Setup skip link
  const skipLink = document.querySelector('.skip-link');
  if (skipLink) {
    skipLink.addEventListener('click', event => {
      event.preventDefault();
      const target = document.querySelector(skipLink.getAttribute('href'));
      if (target) {
        target.focus();
        target.scrollIntoView({ behavior: 'smooth' });
      }
    });
  }

  // Announce page load
  announce('Page chargée', 'polite');
}
