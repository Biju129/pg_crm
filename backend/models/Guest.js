const mongoose = require('mongoose');

const guestSchema = new mongoose.Schema(
  {
    name: {
      type: String,
      required: true,
      trim: true
    },
    roomNumber: {
      type: String,
      required: true,
      trim: true
    },
    phone: {
      type: String,
      trim: true
    },
    checkInDate: {
      type: Date,
      required: true,
      default: Date.now
    },
    checkOutDate: {
      type: Date
    },
    amountDue: {
      type: Number,
      default: 0
    }
  },
  {
    timestamps: true
  }
);

const Guest = mongoose.model('Guest', guestSchema);

module.exports = Guest;