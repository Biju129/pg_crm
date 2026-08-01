const express = require('express');
const Guest = require('../models/Guest');
const router = express.Router();

// CREATE – add a new guest
router.post('/', async (req, res) => {
  try {
    const guest = await Guest.create(req.body);
    res.status(201).json(guest);
  } catch (err) {
    res.status(400).json({ error: err.message });
  }
});

// READ ALL – get all guests
router.get('/', async (req, res) => {
  try {
    const guests = await Guest.find().sort({ createdAt: -1 });
    res.json(guests);
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

// READ ONE – get guest by id
router.get('/:id', async (req, res) => {
  try {
    const guest = await Guest.findById(req.params.id);
    if (!guest) {
      return res.status(404).json({ error: 'Guest not found' });
    }
    res.json(guest);
  } catch (err) {
    res.status(400).json({ error: 'Invalid ID' });
  }
});

// UPDATE – update guest by id
router.put('/:id', async (req, res) => {
  try {
    const guest = await Guest.findByIdAndUpdate(req.params.id, req.body, {
      new: true,
      runValidators: true
    });
    if (!guest) {
      return res.status(404).json({ error: 'Guest not found' });
    }
    res.json(guest);
  } catch (err) {
    res.status(400).json({ error: err.message });
  }
});

// DELETE – remove guest by id
router.delete('/:id', async (req, res) => {
  try {
    const guest = await Guest.findByIdAndDelete(req.params.id);
    if (!guest) {
      return res.status(404).json({ error: 'Guest not found' });
    }
    res.json({ message: 'Guest deleted' });
  } catch (err) {
    res.status(400).json({ error: 'Invalid ID' });
  }
});

module.exports = router;