const API_BASE = '/api';

// State Store
let state = {
  rooms: [],
  tenants: [],
  ledger: [],
  receipts: [],
  notifications: [],
  activeTab: 'dashboard'
};

// Initialize Application
document.addEventListener('DOMContentLoaded', () => {
  initNavigation();
  loadAllData();
  // Auto-refresh every 30 seconds
  setInterval(loadAllData, 30000);
});

// Navigation Setup
function initNavigation() {
  const links = document.querySelectorAll('.nav-link');
  links.forEach(link => {
    link.addEventListener('click', (e) => {
      e.preventDefault();
      const tab = link.getAttribute('data-tab');
      switchTab(tab);
    });
  });
}

function switchTab(tabId) {
  state.activeTab = tabId;
  
  // Update nav links
  document.querySelectorAll('.nav-link').forEach(link => {
    if (link.getAttribute('data-tab') === tabId) {
      link.classList.add('active');
    } else {
      link.classList.remove('active');
    }
  });

  // Update tab panes
  document.querySelectorAll('.tab-pane').forEach(pane => {
    if (pane.id === `tab-${tabId}`) {
      pane.classList.add('active');
    } else {
      pane.classList.remove('active');
    }
  });

  // Update Header Titles
  const titles = {
    dashboard: ['Dashboard Overview', 'Real-time occupancy, revenue tracking, and automated reminders'],
    rooms: ['Rooms & Space Allocation', 'Manage PG room capacity, floor assignments, and occupancy'],
    enrollment: ['Tenant Enrollment & Registry', 'Enroll new tenants, record advance & first rent, verify activation'],
    ledger: ['Monthly Rent Ledger', 'Track monthly billing due dates, payments, and overdue accounts'],
    receipts: ['Receipts Registry', 'View and print official payment receipts'],
    reminders: ['WhatsApp Reminder Engine', 'Automated rent due reminders, overdue notices, and welcome credentials'],
    security: ['User Accounts & Security', 'Manage administrative accounts and security access']
  };

  if (titles[tabId]) {
    document.getElementById('page-title').textContent = titles[tabId][0];
    document.getElementById('page-subtitle').textContent = titles[tabId][1];
  }
}

// Load All Data
async function loadAllData() {
  try {
    await Promise.all([
      fetchRooms(),
      fetchTenants(),
      fetchLedger(),
      fetchReceipts(),
      fetchNotifications()
    ]);
    renderDashboard();
    renderRooms();
    renderEnrollments();
    renderLedger();
    renderReceipts();
    renderNotifications();
    populateRoomDropdown();
  } catch (err) {
    showToast(`Error loading data: ${err.message}`, 'error');
  }
}

// API Calls
async function fetchRooms() {
  const res = await fetch(`${API_BASE}/rooms`);
  if (res.ok) state.rooms = await res.json();
}

async function fetchTenants() {
  const res = await fetch(`${API_BASE}/tenants`);
  if (res.ok) state.tenants = await res.json();
}

async function fetchLedger() {
  const res = await fetch(`${API_BASE}/ledger`);
  if (res.ok) state.ledger = await res.json();
}

async function fetchReceipts() {
  const res = await fetch(`${API_BASE}/receipts`);
  if (res.ok) state.receipts = await res.json();
}

async function fetchNotifications() {
  const res = await fetch(`${API_BASE}/notifications`);
  if (res.ok) state.notifications = await res.json();
}

// Dashboard Calculations
function renderDashboard() {
  const activeTenants = state.tenants.filter(t => t.enrollment_status === 'ACTIVE').length;
  const pendingTenants = state.tenants.filter(t => t.enrollment_status === 'PENDING_PAYMENT').length;
  
  const totalCapacity = state.rooms.reduce((acc, r) => acc + (r.capacity || 0), 0);
  const availableSpaces = state.rooms.filter(r => r.status === 'AVAILABLE').reduce((acc, r) => acc + (r.capacity || 0), 0);

  const pendingDues = state.ledger.reduce((acc, l) => acc + (l.pending_amount || 0), 0);
  const overdueCount = state.ledger.filter(l => l.payment_status === 'OVERDUE').length;

  const totalRevenue = state.ledger.reduce((acc, l) => acc + (l.amount_paid || 0), 0) +
                       state.tenants.reduce((acc, t) => acc + (t.advance_amount || 0), 0);

  document.getElementById('stat-active-tenants').textContent = activeTenants;
  document.getElementById('stat-pending-enrollments').textContent = `${pendingTenants} pending verification`;

  document.getElementById('stat-room-capacity').textContent = `${state.rooms.length} Rooms / ${totalCapacity} Beds`;
  document.getElementById('stat-available-spaces').textContent = `${availableSpaces} spaces available`;

  document.getElementById('stat-pending-dues').textContent = `₹${pendingDues.toLocaleString('en-IN')}`;
  document.getElementById('stat-overdue-count').textContent = `${overdueCount} overdue items`;

  document.getElementById('stat-revenue').textContent = `₹${totalRevenue.toLocaleString('en-IN')}`;

  // Recent Enrollments Table
  const body = document.getElementById('dash-enrollments-body');
  const recent = state.tenants.slice(0, 5);
  body.innerHTML = recent.map(t => {
    const room = state.rooms.find(r => r.id === t.room_id);
    const roomNo = room ? room.room_number : 'Unassigned';
    return `
      <tr>
        <td><strong>${t.tenant_id || 'PENDING'}</strong></td>
        <td>${t.full_name}</td>
        <td>${t.contact_number}</td>
        <td><span class="badge badge-info">Room ${roomNo}</span></td>
        <td>₹${(t.advance_amount || 0).toLocaleString('en-IN')}</td>
        <td><span class="badge badge-${t.enrollment_status === 'ACTIVE' ? 'success' : 'warning'}">${t.enrollment_status}</span></td>
        <td>
          ${t.enrollment_status === 'PENDING_PAYMENT' 
            ? `<button class="btn btn-sm btn-primary" onclick="openVerifyModal('${t.id}')">Verify Payment</button>`
            : `<span class="text-dim">Verified</span>`}
        </td>
      </tr>
    `;
  }).join('');
}

// Rooms Rendering
function renderRooms() {
  const container = document.getElementById('rooms-grid-container');
  container.innerHTML = state.rooms.map(room => {
    const activeTenantsInRoom = state.tenants.filter(t => t.room_id === room.id && t.enrollment_status === 'ACTIVE').length;
    const isFull = activeTenantsInRoom >= room.capacity;
    const fillPercent = Math.min(100, Math.round((activeTenantsInRoom / room.capacity) * 100));

    return `
      <div class="room-card">
        <div class="room-card-header">
          <div class="room-number">Room ${room.room_number}</div>
          <span class="badge badge-${isFull ? 'danger' : 'success'}">${isFull ? 'FULL' : 'AVAILABLE'}</span>
        </div>
        <div style="font-size: 0.85rem; color: var(--text-muted);">Floor ${room.floor_number} • Capacity: ${room.capacity} Bed(s)</div>
        <div class="occupancy-bar">
          <div class="occupancy-fill" style="width: ${fillPercent}%;"></div>
        </div>
        <div style="display:flex; justify-content:space-between; font-size: 0.8rem; margin-top: 8px;">
          <span>Occupancy: <strong>${activeTenantsInRoom} / ${room.capacity}</strong></span>
          <span style="color: var(--accent-cyan); font-weight:600;">₹${room.monthly_rent}/mo</span>
        </div>
      </div>
    `;
  }).join('');
}

function populateRoomDropdown() {
  const select = document.getElementById('enroll-room-id');
  const available = state.rooms.filter(r => r.status === 'AVAILABLE');
  select.innerHTML = available.length === 0 
    ? `<option value="">No available rooms</option>`
    : available.map(r => `<option value="${r.id}" data-rent="${r.monthly_rent}">Room ${r.room_number} (Floor ${r.floor_number}) — ₹${r.monthly_rent}/mo</option>`).join('');

  onRoomSelectChange();
}

function onRoomSelectChange() {
  const select = document.getElementById('enroll-room-id');
  const selectedOption = select.options[select.selectedIndex];
  if (selectedOption && selectedOption.dataset.rent) {
    document.getElementById('enroll-rent').value = selectedOption.dataset.rent;
    calcTotalPayable();
  }
}

function calcTotalPayable() {
  const rent = parseFloat(document.getElementById('enroll-rent').value) || 0;
  const advance = parseFloat(document.getElementById('enroll-advance').value) || 0;
  const total = rent + advance;
  document.getElementById('enroll-total-payable').textContent = `₹${total.toLocaleString('en-IN')}.00`;
}

// Tenant Enrollment Form
async function handleTenantEnrollment(e) {
  e.preventDefault();
  const dto = {
    full_name: document.getElementById('enroll-fullname').value,
    contact_number: document.getElementById('enroll-contact').value,
    email: document.getElementById('enroll-email').value || null,
    joining_date: document.getElementById('enroll-date').value || new Date().toISOString().split('T')[0],
    room_id: document.getElementById('enroll-room-id').value,
    occupation_type: document.getElementById('enroll-occupation').value,
    organization_name: document.getElementById('enroll-org').value || null,
    monthly_rent: parseFloat(document.getElementById('enroll-rent').value),
    advance_amount: parseFloat(document.getElementById('enroll-advance').value)
  };

  try {
    const res = await fetch(`${API_BASE}/tenants`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(dto)
    });
    if (!res.ok) {
      const err = await res.json();
      throw new Error(err.error || 'Failed to enroll tenant');
    }
    showToast('Tenant enrollment created! Now record initial payments.', 'success');
    document.getElementById('enrollment-form').reset();
    await loadAllData();
  } catch (err) {
    showToast(err.message, 'error');
  }
}

function renderEnrollments() {
  const body = document.getElementById('enrollment-table-body');
  body.innerHTML = state.tenants.map(t => {
    const room = state.rooms.find(r => r.id === t.room_id);
    const roomNo = room ? room.room_number : 'N/A';
    const totalPayable = (t.advance_amount || 0) + (t.monthly_rent || 0);

    return `
      <tr>
        <td>
          <strong>${t.full_name}</strong><br>
          <span style="font-size:0.75rem; color:var(--accent-cyan);">${t.tenant_id || 'Pending ID'}</span>
        </td>
        <td>${t.contact_number}</td>
        <td><span class="badge badge-info">Room ${roomNo}</span></td>
        <td>₹${totalPayable.toLocaleString('en-IN')}</td>
        <td><span class="badge badge-${t.enrollment_status === 'ACTIVE' ? 'success' : 'warning'}">${t.enrollment_status}</span></td>
        <td>
          ${t.enrollment_status === 'PENDING_PAYMENT'
            ? `<button class="btn btn-sm btn-primary" onclick="openVerifyModal('${t.id}')">Verify & Activate</button>`
            : `<span class="badge badge-success">Activated</span>`}
        </td>
      </tr>
    `;
  }).join('');
}

// Verification Modal
async function openVerifyModal(tenantId) {
  try {
    const res = await fetch(`${API_BASE}/tenants/${tenantId}`);
    if (!res.ok) throw new Error('Failed to fetch tenant details');
    const detail = await res.json();
    
    const body = document.getElementById('verify-modal-body');
    const t = detail.tenant;
    const payments = detail.enrollment_payments || [];

    const advancePayment = payments.find(p => p.payment_type === 'ADVANCE');
    const rentPayment = payments.find(p => p.payment_type === 'FIRST_MONTH_RENT');

    body.innerHTML = `
      <div style="margin-bottom:16px;">
        <h4 style="font-size:1.1rem; color:var(--accent-cyan);">${t.full_name}</h4>
        <p style="font-size:0.85rem; color:var(--text-muted);">Contact: ${t.contact_number} • Room ID: ${t.room_id}</p>
      </div>

      <div class="card" style="padding:14px; margin-bottom:14px;">
        <h5 style="margin-bottom:8px;">1. Advance Payment (Refundable)</h5>
        <div style="display:flex; justify-content:space-between; align-items:center;">
          <span>Amount Due: <strong>₹${t.advance_amount}</strong></span>
          <span class="badge badge-${advancePayment?.payment_status === 'PAID' ? 'success' : 'warning'}">${advancePayment?.payment_status || 'PENDING'}</span>
        </div>
        ${advancePayment?.payment_status !== 'PAID' ? `
          <button class="btn btn-sm btn-secondary" style="margin-top:8px;" onclick="recordEnrollmentPayment('${t.id}', 'ADVANCE', ${t.advance_amount})">Mark Advance as PAID</button>
        ` : ''}
      </div>

      <div class="card" style="padding:14px; margin-bottom:14px;">
        <h5 style="margin-bottom:8px;">2. First Month Rent</h5>
        <div style="display:flex; justify-content:space-between; align-items:center;">
          <span>Amount Due: <strong>₹${t.monthly_rent}</strong></span>
          <span class="badge badge-${rentPayment?.payment_status === 'PAID' ? 'success' : 'warning'}">${rentPayment?.payment_status || 'PENDING'}</span>
        </div>
        ${rentPayment?.payment_status !== 'PAID' ? `
          <button class="btn btn-sm btn-secondary" style="margin-top:8px;" onclick="recordEnrollmentPayment('${t.id}', 'FIRST_MONTH_RENT', ${t.monthly_rent})">Mark First Month Rent as PAID</button>
        ` : ''}
      </div>

      <button class="btn btn-primary btn-block" onclick="activateTenant('${t.id}')">Complete Verification & Activate Account</button>
    `;

    document.getElementById('verify-modal').classList.add('active');
  } catch (err) {
    showToast(err.message, 'error');
  }
}

async function recordEnrollmentPayment(tenantId, paymentType, amount) {
  try {
    const res = await fetch(`${API_BASE}/tenants/${tenantId}/payments`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        tenant_id: tenantId,
        payment_type: paymentType,
        amount_paid: amount,
        payment_method: 'CASH',
        reference_id: `CASH-${Date.now()}`
      })
    });
    if (!res.ok) throw new Error('Failed to record payment');
    showToast(`${paymentType} marked as PAID`, 'success');
    await openVerifyModal(tenantId);
  } catch (err) {
    showToast(err.message, 'error');
  }
}

async function activateTenant(tenantId) {
  try {
    const res = await fetch(`${API_BASE}/tenants/${tenantId}/verify`, {
      method: 'POST'
    });
    if (!res.ok) {
      const err = await res.json();
      throw new Error(err.error || 'Verification failed. Ensure both payments are marked PAID.');
    }
    const result = await res.json();
    showToast(`Tenant Activated! Assigned ID: ${result.login_username.toUpperCase()} (Temp Pass: ${result.temporary_password})`, 'success');
    closeModal('verify-modal');
    await loadAllData();
  } catch (err) {
    showToast(err.message, 'error');
  }
}

// Render Ledger
function renderLedger() {
  const body = document.getElementById('ledger-table-body');
  body.innerHTML = state.ledger.map(l => {
    const tenant = state.tenants.find(t => t.id === l.tenant_id);
    const tenantName = tenant ? tenant.full_name : 'Unknown';
    const tenantCode = tenant?.tenant_id || 'TNT';

    return `
      <tr>
        <td><strong>${tenantCode}</strong><br><span style="font-size:0.8rem; color:var(--text-muted);">${tenantName}</span></td>
        <td>${l.billing_month}</td>
        <td>${l.due_date}</td>
        <td>₹${l.rent_due.toLocaleString('en-IN')}</td>
        <td>₹${l.amount_paid.toLocaleString('en-IN')}</td>
        <td style="color: ${l.pending_amount > 0 ? 'var(--accent-amber)' : 'var(--text-muted)'}">₹${l.pending_amount.toLocaleString('en-IN')}</td>
        <td><span class="badge badge-${l.payment_status === 'PAID' ? 'success' : (l.payment_status === 'OVERDUE' ? 'danger' : 'warning')}">${l.payment_status}</span></td>
        <td>
          ${l.payment_status !== 'PAID' 
            ? `<button class="btn btn-sm btn-primary" onclick="openPayRentModal('${l.id}', ${l.pending_amount})">Pay Rent</button>`
            : `<span class="badge badge-success">Cleared</span>`}
        </td>
      </tr>
    `;
  }).join('');
}

function openPayRentModal(ledgerId, pendingAmount) {
  document.getElementById('pay-ledger-id').value = ledgerId;
  document.getElementById('pay-amount').value = pendingAmount;
  document.getElementById('pay-rent-modal').classList.add('active');
}

async function handlePayRent(e) {
  e.preventDefault();
  const dto = {
    ledger_id: document.getElementById('pay-ledger-id').value,
    amount_paid: parseFloat(document.getElementById('pay-amount').value),
    payment_method: document.getElementById('pay-method').value,
    issued_by: null
  };

  try {
    const res = await fetch(`${API_BASE}/ledger/pay`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(dto)
    });
    if (!res.ok) {
      const err = await res.json();
      throw new Error(err.error || 'Payment failed');
    }
    const data = await res.json();
    showToast(`Payment of ₹${dto.amount_paid} recorded! Receipt: ${data.receipt.receipt_number}`, 'success');
    closeModal('pay-rent-modal');
    await loadAllData();
    showReceiptModal(data.receipt);
  } catch (err) {
    showToast(err.message, 'error');
  }
}

// Receipts
function renderReceipts() {
  const body = document.getElementById('receipts-table-body');
  body.innerHTML = state.receipts.map(r => {
    const tenant = state.tenants.find(t => t.id === r.tenant_id);
    const tenantName = tenant ? tenant.full_name : 'Tenant';
    return `
      <tr>
        <td><strong style="color:var(--accent-cyan);">${r.receipt_number}</strong></td>
        <td>${tenantName}</td>
        <td><span class="badge badge-info">${r.payment_method}</span></td>
        <td>₹${r.amount.toLocaleString('en-IN')}</td>
        <td>${new Date(r.issued_at).toLocaleDateString()}</td>
        <td>
          <button class="btn btn-sm btn-secondary" onclick="viewReceipt('${r.id}')">View Receipt</button>
        </td>
      </tr>
    `;
  }).join('');
}

async function viewReceipt(receiptId) {
  const r = state.receipts.find(item => item.id === receiptId);
  if (r) showReceiptModal(r);
}

function showReceiptModal(receipt) {
  const tenant = state.tenants.find(t => t.id === receipt.tenant_id);
  const body = document.getElementById('receipt-modal-body');

  body.innerHTML = `
    <div style="text-align:center; padding:10px 0; border-bottom:1px solid #ccc; margin-bottom:15px;">
      <h2 style="margin:0; color:#000;">OFFICIAL PAYMENT RECEIPT</h2>
      <p style="margin:0; color:#666; font-size:0.85rem;">Project RAM PG Management Suite</p>
    </div>

    <div style="display:flex; justify-content:space-between; margin-bottom:12px; color:#000;">
      <div>
        <strong>Receipt No:</strong> ${receipt.receipt_number}<br>
        <strong>Issued Date:</strong> ${new Date(receipt.issued_at).toLocaleString()}
      </div>
      <div>
        <strong>Payment Mode:</strong> ${receipt.payment_method}
      </div>
    </div>

    <div style="background:#f9f9f9; padding:12px; border-radius:6px; margin-bottom:15px; color:#000;">
      <strong>Tenant Details:</strong><br>
      Name: ${tenant?.full_name || 'N/A'}<br>
      Tenant ID: ${tenant?.tenant_id || 'N/A'}<br>
      Contact: ${tenant?.contact_number || 'N/A'}
    </div>

    <table style="width:100%; border-collapse:collapse; color:#000; margin-bottom:20px;">
      <tr style="border-bottom:1px solid #ccc;">
        <th style="text-align:left; padding:6px;">Description</th>
        <th style="text-align:right; padding:6px;">Amount</th>
      </tr>
      <tr>
        <td style="padding:6px;">Monthly Rent Payment</td>
        <td style="text-align:right; padding:6px;">₹${receipt.amount.toLocaleString('en-IN')}.00</td>
      </tr>
      <tr style="font-weight:bold; border-top:1px solid #000;">
        <td style="padding:6px;">Total Paid</td>
        <td style="text-align:right; padding:6px;">₹${receipt.amount.toLocaleString('en-IN')}.00</td>
      </tr>
    </table>

    <div style="text-align:center; font-size:0.8rem; color:#666;">
      Thank you for your payment! This is a system-generated electronic receipt.
    </div>
  `;

  document.getElementById('receipt-modal').classList.add('active');
}

// Notifications
function renderNotifications() {
  const body = document.getElementById('notifications-table-body');
  body.innerHTML = state.notifications.map(n => {
    return `
      <tr>
        <td><span class="badge badge-info">${n.notification_type}</span></td>
        <td>${n.channel}</td>
        <td style="font-family:monospace; font-size:0.8rem;">${n.message_reference || '—'}</td>
        <td><span class="badge badge-${n.status === 'SENT' ? 'success' : 'warning'}">${n.status}</span></td>
        <td>${new Date(n.created_at).toLocaleString()}</td>
      </tr>
    `;
  }).join('');
}

async function runReminderCycle() {
  try {
    const res = await fetch(`${API_BASE}/reminders/run`, { method: 'POST' });
    if (!res.ok) throw new Error('Reminder engine failed');
    const summary = await res.json();
    showToast(`Reminder Engine Completed! Marked Overdue: ${summary.overdue_marked}, Due Queued: ${summary.due_reminders_queued}, Overdue Queued: ${summary.overdue_reminders_queued}, Processed: ${summary.notifications_processed}`, 'success');
    await loadAllData();
  } catch (err) {
    showToast(err.message, 'error');
  }
}

// Modals Helper
function closeModal(id) {
  document.getElementById(id).classList.remove('active');
}

function openRoomModal() {
  document.getElementById('room-form').reset();
  document.getElementById('room-modal-id').value = '';
  document.getElementById('room-modal').classList.add('active');
}

async function handleSaveRoom(e) {
  e.preventDefault();
  const dto = {
    room_number: document.getElementById('room-modal-number').value,
    floor_number: parseInt(document.getElementById('room-modal-floor').value),
    capacity: parseInt(document.getElementById('room-modal-capacity').value),
    monthly_rent: parseFloat(document.getElementById('room-modal-rent').value)
  };

  try {
    const res = await fetch(`${API_BASE}/rooms`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(dto)
    });
    if (!res.ok) throw new Error('Failed to create room');
    showToast('Room created successfully!', 'success');
    closeModal('room-modal');
    await loadAllData();
  } catch (err) {
    showToast(err.message, 'error');
  }
}

// Toast System
function showToast(msg, type = 'info') {
  const container = document.getElementById('toast-container');
  const toast = document.createElement('div');
  toast.className = 'toast';
  toast.style.borderColor = type === 'success' ? 'var(--accent-emerald)' : (type === 'error' ? 'var(--accent-rose)' : 'var(--accent-cyan)');
  toast.textContent = msg;
  container.appendChild(toast);
  setTimeout(() => toast.remove(), 4000);
}
