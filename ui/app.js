// Desktop UI Application Script
const API_URL = 'http://localhost:8080/api';

// State Store
let guests = [];
let rooms = [];

// DOM Elements
const guestForm = document.getElementById('guest-form');
const guestIdInput = document.getElementById('guest-id');
const nameInput = document.getElementById('name');
const roomInput = document.getElementById('roomNumber');
const phoneInput = document.getElementById('phone');
const rentInput = document.getElementById('monthlyRent');
const advanceInput = document.getElementById('advanceAmount');
const amountInput = document.getElementById('amountDue');
const formTitle = document.getElementById('form-title');
const saveBtn = document.getElementById('save-btn');
const cancelBtn = document.getElementById('cancel-btn');
const guestTableBody = document.getElementById('guest-table-body');

const roomForm = document.getElementById('room-form');
const roomNumberInput = document.getElementById('room-number');
const roomCapacityInput = document.getElementById('room-capacity');
const roomTableBody = document.getElementById('room-table-body');

const userForm = document.getElementById('user-form');

// Stats Elements
const statGuests = document.getElementById('stat-total-guests');
const statRooms = document.getElementById('stat-active-rooms');
const statDue = document.getElementById('stat-total-due');

// Tab Navigation
document.querySelectorAll('.nav-item').forEach(item => {
  item.addEventListener('click', () => {
    document.querySelectorAll('.nav-item').forEach(n => n.classList.remove('active'));
    document.querySelectorAll('.tab-content').forEach(t => t.style.display = 'none');

    item.classList.add('active');
    const tabName = item.getAttribute('data-tab');
    document.getElementById(`tab-${tabName}`).style.display = 'block';

    const titles = {
      guests: 'Guest Dashboard',
      rooms: 'Rooms & Occupancy',
      auth: 'Users & Security'
    };
    document.getElementById('page-title').innerText = titles[tabName] || 'Dashboard';
  });
});

// Load Guests & Rooms Data
async function loadData() {
  await Promise.all([fetchGuests(), fetchRooms()]);
  updateStats();
}

async function fetchGuests() {
  try {
    const res = await fetch(`${API_URL}/guests`);
    if (res.ok) {
      guests = await res.json();
      renderGuests();
    }
  } catch (err) {
    console.warn('API Offline or database connecting. Demo mode active.');
  }
}

async function fetchRooms() {
  try {
    const res = await fetch(`${API_URL}/rooms`);
    if (res.ok) {
      rooms = await res.json();
      renderRooms();
    }
  } catch (err) {
    console.warn('API Offline or database connecting.');
  }
}

function renderGuests() {
  guestTableBody.innerHTML = '';
  if (guests.length === 0) {
    guestTableBody.innerHTML = `<tr><td colspan="6" style="text-align:center; color:var(--text-muted);">No guests added yet</td></tr>`;
    return;
  }

  guests.forEach(g => {
    const tr = document.createElement('tr');
    const dueClass = g.amount_due > 0 ? 'badge-due' : 'badge-clear';
    const dueText = g.amount_due > 0 ? `₹${g.amount_due.toFixed(2)}` : 'Cleared';

    tr.innerHTML = `
      <td><strong>${escapeHtml(g.name)}</strong></td>
      <td>${escapeHtml(g.room_number)}</td>
      <td>${escapeHtml(g.phone || '-')}</td>
      <td>₹${(g.monthly_rent || 0).toFixed(2)}</td>
      <td><span class="badge ${dueClass}">${dueText}</span></td>
      <td>
        <button class="btn btn-edit" onclick="editGuest('${g.id}')">Edit</button>
        <button class="btn btn-danger" onclick="deleteGuest('${g.id}')">Delete</button>
      </td>
    `;
    guestTableBody.appendChild(tr);
  });
}

function renderRooms() {
  roomTableBody.innerHTML = '';
  if (rooms.length === 0) {
    roomTableBody.innerHTML = `<tr><td colspan="5" style="text-align:center; color:var(--text-muted);">No rooms configured yet</td></tr>`;
    return;
  }

  rooms.forEach(r => {
    const tr = document.createElement('tr');
    const isFull = r.occupied >= r.capacity;
    const statusBadge = isFull ? `<span class="badge badge-due">Full</span>` : `<span class="badge badge-clear">Available</span>`;

    tr.innerHTML = `
      <td><strong>Room ${escapeHtml(r.room_number)}</strong></td>
      <td>${r.capacity} Beds</td>
      <td>${r.occupied} Occupied</td>
      <td>${statusBadge}</td>
      <td>
        <button class="btn btn-danger" onclick="deleteRoom('${r.id}')">Delete</button>
      </td>
    `;
    roomTableBody.appendChild(tr);
  });
}

function updateStats() {
  statGuests.innerText = guests.length;
  statRooms.innerText = rooms.length;
  const totalDue = guests.reduce((sum, g) => sum + (g.amount_due || 0), 0);
  statDue.innerText = `₹${totalDue.toFixed(2)}`;
}

// Guest Form Actions
guestForm.addEventListener('submit', async (e) => {
  e.preventDefault();
  const guestData = {
    name: nameInput.value.trim(),
    room_number: roomInput.value.trim(),
    phone: phoneInput.value.trim() || null,
    monthly_rent: parseFloat(rentInput.value) || 0,
    advance_amount: parseFloat(advanceInput.value) || 0,
    amount_due: parseFloat(amountInput.value) || 0
  };

  const id = guestIdInput.value;

  try {
    if (id) {
      await fetch(`${API_URL}/guests/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(guestData)
      });
    } else {
      await fetch(`${API_URL}/guests`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(guestData)
      });
    }
  } catch (err) {
    console.error('Save failed:', err);
  }

  resetGuestForm();
  loadData();
});

window.editGuest = function(id) {
  const g = guests.find(item => item.id === id);
  if (!g) return;

  guestIdInput.value = g.id;
  nameInput.value = g.name;
  roomInput.value = g.room_number;
  phoneInput.value = g.phone || '';
  rentInput.value = g.monthly_rent || 0;
  advanceInput.value = g.advance_amount || 0;
  amountInput.value = g.amount_due || 0;

  formTitle.innerText = 'Edit Guest Details';
  saveBtn.innerText = 'Update Guest';
  cancelBtn.style.display = 'inline-block';
};

window.deleteGuest = async function(id) {
  if (!confirm('Are you sure you want to delete this guest?')) return;
  try {
    await fetch(`${API_URL}/guests/${id}`, { method: 'DELETE' });
  } catch (err) {
    console.error('Delete failed:', err);
  }
  loadData();
};

function resetGuestForm() {
  guestIdInput.value = '';
  guestForm.reset();
  formTitle.innerText = 'Add New Guest';
  saveBtn.innerText = 'Save Guest';
  cancelBtn.style.display = 'none';
}

cancelBtn.addEventListener('click', resetGuestForm);

// Room Form Actions
roomForm.addEventListener('submit', async (e) => {
  e.preventDefault();
  const roomData = {
    room_number: roomNumberInput.value.trim(),
    capacity: parseInt(roomCapacityInput.value) || 1
  };

  try {
    await fetch(`${API_URL}/rooms`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(roomData)
    });
  } catch (err) {
    console.error('Save room failed:', err);
  }

  roomForm.reset();
  loadData();
});

window.deleteRoom = async function(id) {
  if (!confirm('Delete this room?')) return;
  try {
    await fetch(`${API_URL}/rooms/${id}`, { method: 'DELETE' });
  } catch (err) {
    console.error('Delete room failed:', err);
  }
  loadData();
};

// User Registration Form
userForm.addEventListener('submit', async (e) => {
  e.preventDefault();
  const userData = {
    name: document.getElementById('user-name').value,
    email: document.getElementById('user-email').value,
    password: document.getElementById('user-password').value,
    role: document.getElementById('user-role').value
  };

  try {
    const res = await fetch(`${API_URL}/auth/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(userData)
    });
    if (res.ok) {
      alert('User created successfully!');
      userForm.reset();
    } else {
      const err = await res.json();
      alert(`Error: ${err.error || 'Failed to create user'}`);
    }
  } catch (err) {
    alert('User registration submitted');
    userForm.reset();
  }
});

function escapeHtml(str) {
  return (str || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

// Initial Load
loadData();
