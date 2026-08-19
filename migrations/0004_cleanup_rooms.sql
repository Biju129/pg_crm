-- Migration 0004: Align rooms table with Project RAM V1 (remove legacy columns)

DO $$ BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'rooms' AND column_name = 'occupied'
    ) THEN
        ALTER TABLE rooms DROP COLUMN occupied;
    END IF;
END $$;

DO $$ BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'rooms' AND column_name = 'monthly_rent_rate'
    ) THEN
        IF NOT EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_name = 'rooms' AND column_name = 'monthly_rent'
        ) THEN
            ALTER TABLE rooms RENAME COLUMN monthly_rent_rate TO monthly_rent;
        ELSE
            UPDATE rooms SET monthly_rent = monthly_rent_rate WHERE monthly_rent = 0;
            ALTER TABLE rooms DROP COLUMN monthly_rent_rate;
        END IF;
    END IF;
END $$;

SELECT 'Migration 0004 complete: rooms table cleaned up.' AS result;
