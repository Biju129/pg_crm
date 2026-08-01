const express = require('express');
const cors = require('cors');
const connectDB = require('./config/db');
const guestRoutes = require('./routes/guestRoutes');
const authRoutes = require('./routes/authRoutes');
// Connect to MongoDB
connectDB();

const app = express();

// Middleware
app.use(cors());
app.use(express.json());

// Routes
app.use('/api/guests', guestRoutes);
app.use('/api/auth', authRoutes);
// Basic health check route (optional)
app.get('/', (req, res) => {
  res.send('PG CRM API is running');
});

// Start server
const PORT = 5000;
app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});