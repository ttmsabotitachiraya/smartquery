# SmartQuery for HOSxP — v3.0.0

> ระบบตรวจสอบความถูกต้องของข้อมูลและรายงาน SQL สำหรับฐานข้อมูล HOSxP (MariaDB/MySQL)
> พัฒนาโดย: กลุ่มงานแพทย์แผนไทย โรงพยาบาลสระโบสถ์ ลพบุรี

---

## Tech Stack

| Layer       | Technology                              |
|-------------|----------------------------------------|
| Desktop     | [Tauri 2](https://tauri.app/) (Rust)    |
| Frontend    | [Vue 3](https://vuejs.org/) + TypeScript |
| Styling     | [Tailwind CSS v4](https://tailwindcss.com/) |
| Icons       | [Lucide Vue Next](https://lucide.dev/)  |
| DB (HOSxP)  | MySQL/MariaDB via `sqlx` (Rust crate)   |
| DB (Local)  | SQLite via `rusqlite` (bundled)          |

---

## Features

### 🔍 Audit Page
- รันคำสั่ง SQL ตรวจสอบข้อมูลหลายรายการพร้อมกัน
- เลือกช่วงวันที่ (ปีงบประมาณไทย ต.ค. – ก.ย.)
- Run All / Run Selected / Run Starred
- แสดงสถานะ Idle / Running / Pass / Error / Stopped
- ดูผลลัพธ์ในตาราง + Export CSV (UTF-8 BOM)
- กรองด้วยชื่อ, กลุ่ม, สถานะ

### 📊 Reports Page
- เลือกรายงานจากรายการ → ตั้งช่วงวันที่ → Run
- กรองผลลัพธ์แบบ real-time
- Export CSV

### ⚙️ SQL Management
- CRUD สำหรับ Audit Queries และ Report Queries
- Toggle เปิด/ปิด Query
- Starred/Favourite
- ทดสอบ SQL ก่อนบันทึก
- จัดการ Groups

### 🕒 History
- บันทึกประวัติการรัน Query ทุกครั้ง
- กรองตาม Mode / Status
- ลบประวัติ

### 🔧 Settings
- ตั้งค่าเชื่อมต่อ HOSxP (Host, Port, User, Password, Database)
- ทดสอบการเชื่อมต่อ
- **นำเข้าข้อมูลจากโปรแกรมเดิม (Python)** — ไฟล์ `sql_storage.db`

---

## Color Theme

| Element         | Hex       | ความหมาย                          |
|-----------------|-----------|-----------------------------------|
| Background      | `#2B2B2B` | Deep Charcoal — พื้นหลังหลัก      |
| Surface/Sidebar | `#3D3D3D` | สีเตาเหล็ก — เมนู/แถบเครื่องมือ  |
| Primary Accent  | `#E63946` | แดง — ปุ่มหลัก / Run              |
| Secondary       | `#FFB703` | เหลือง — Icon / Warning           |
| Success         | `#80B918` | เขียว — Pass / เชื่อมต่อสำเร็จ   |
| Text Primary    | `#F8EDEB` | ขาวนวล — ข้อความทั่วไป            |

---

## Prerequisites

```
Node.js >= 18
Rust (stable) — https://rustup.rs/
```

### macOS dependencies
```bash
xcode-select --install
```

### Ubuntu/Debian dependencies
```bash
sudo apt install -y libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

---

## Development

```bash
# ติดตั้ง dependencies
cd smartquery-tauri
npm install

# รันในโหมด development (Hot-Reload)
npm run tauri dev
```

---

## Build (Production)

```bash
cd smartquery-tauri
npm run tauri build
```

ไฟล์ที่ได้จะอยู่ใน `src-tauri/target/release/bundle/`

---

## Project Structure

```
smartquery-tauri/
├── src/                        # Vue 3 Frontend
│   ├── components/
│   │   ├── AuditPage.vue       # หน้า Audit
│   │   ├── ReportPage.vue      # หน้า Reports
│   │   ├── ManagementPage.vue  # หน้าจัดการ SQL
│   │   ├── SettingsPage.vue    # หน้าตั้งค่า
│   │   ├── HistoryPage.vue     # หน้าประวัติ
│   │   ├── ResultModal.vue     # Modal แสดงผลลัพธ์
│   │   ├── QueryEditorModal.vue# Modal แก้ไข SQL
│   │   ├── GroupsModal.vue     # Modal จัดการกลุ่ม
│   │   └── NavBtn.vue          # Navigation Button
│   ├── App.vue                 # Root component + Sidebar
│   ├── main.ts                 # App entry point
│   ├── styles.css              # Tailwind + custom classes
│   ├── types.ts                # TypeScript interfaces
│   └── tauri-api.ts            # Tauri command wrappers
├── src-tauri/                  # Rust Backend
│   ├── src/
│   │   ├── lib.rs              # Tauri commands entry point
│   │   ├── db_hosxp.rs         # HOSxP MySQL connection (sqlx)
│   │   └── db_local.rs         # SQLite local storage (rusqlite)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── capabilities/
│       └── default.json        # Tauri permissions
└── public/
    └── logo.svg                # App logo
```

---

## Local Database

แอปพลิเคชันจะสร้างไฟล์ SQLite ที่:

| OS      | Path |
|---------|------|
| macOS   | `~/Library/Application Support/SmartQuery/sql_storage.db` |
| Windows | `%APPDATA%\SmartQuery\sql_storage.db` |
| Linux   | `~/.local/share/SmartQuery/sql_storage.db` |

---

## Migration จากเวอร์ชัน Python

1. เปิดแอป → ไปที่หน้า **Settings**
2. เลื่อนลงไปที่ส่วน **"นำเข้าข้อมูลจากโปรแกรมเดิม"**
3. กด **Browse** แล้วเลือกไฟล์ `sql_storage.db` จากโฟลเดอร์โปรแกรม Python เดิม
4. กด **Import Data**

ระบบจะนำเข้า:
- กลุ่ม (Groups)
- คำสั่ง SQL ทั้งหมด (Audit + Report)
- การตั้งค่าการเชื่อมต่อ HOSxP

---

## SQL Safety

ระบบตรวจสอบและอนุญาตเฉพาะคำสั่ง `SELECT` เท่านั้น คำสั่งที่ถูกบล็อก:

`INSERT`, `UPDATE`, `DELETE`, `DROP`, `TRUNCATE`, `ALTER`, `CREATE`,
`REPLACE`, `MERGE`, `CALL`, `EXEC`, `EXECUTE`, `GRANT`, `REVOKE`, `LOCK`

---

## License

Internal use — กลุ่มงานแพทย์แผนไทย โรงพยาบาลสระโบสถ์ ลพบุรี
```

Now let me do a final verification that everything is in place: