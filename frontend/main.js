const API_URL = 'http://localhost:5000/api/guests';

const guestForm = document.getElementById('guest-form');
const guestIdInput = document.getElementById('guest-id');
const nameInput = document.getElementById('name');
const roomInput = document.getElementById('roomNumber');
const phoneInput = document.getElementById('phone');
const amountInput = document.getElementById('amountDue');
const cancelEditBtn = document.getElementById('cancel-edit');
const tableBody = document.getElementById('guest-table-body');

async function loadGuests() {
  tableBody.innerHTML = '';
  const res = await fetch(API_URL);
  const guests = await res.json();

  guests.forEach(guest => {
    const tr = document.createElement('tr');

    tr.innerHTML = `
      <td>${guest.name}</td>
      <td>${guest.roomNumber}</td>
      <td>${guest.phone || ''}</td>
      <td>${guest.amountDue || 0}</td>
      <td>
        <button onclick="editGuest('${guest._id}')">Edit</button>
        <button onclick="deleteGuest('${guest._id}')">Delete</button>
      </td>
    `;

    tableBody.appendChild(tr);
  });
}

guestForm.addEventListener('submit', async (e) => {
  e.preventDefault();

  const data = {
    name: nameInput.value,
    roomNumber: roomInput.value,
    phone: phoneInput.value,
    amountDue: Number(amountInput.value) || 0
  };

  const id = guestIdInput.value;

  if (id) {
    // UPDATE
    await fetch(`${API_URL}/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data)
    });
  } else {
    // CREATE
    await fetch(API_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data)
    });
  }

  resetForm();
  loadGuests();
});

window.editGuest = async function (id) {
  const res = await fetch(`${API_URL}/${id}`);
  const guest = await res.json();

  guestIdInput.value = guest._id;
  nameInput.value = guest.name;
  roomInput.value = guest.roomNumber;
  phoneInput.value = guest.phone || '';
  amountInput.value = guest.amountDue || 0;

  cancelEditBtn.style.display = 'inline-block';
};

window.deleteGuest = async function (id) {
  if (!confirm('Delete this guest?')) return;

  await fetch(`${API_URL}/${id}`, {
    method: 'DELETE'
  });

  loadGuests();
};

function resetForm() {
  guestIdInput.value = '';
  guestForm.reset();
  amountInput.value = 0;
  cancelEditBtn.style.display = 'none';
}

cancelEditBtn.addEventListener('click', resetForm);

// Load data on page load
loadGuests();