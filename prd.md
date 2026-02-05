Створю детальний PRD (Product Requirements Document) для AI-агента, щоб згенерувати код програми. Назва програми LogiBat

***

# PRD: LogiTray - Кросс-платформенний індикатор заряду батареї Logitech

## 1. ОГЛЯД ПРОЄКТУ

**Назва:** LogiTray  
**Ціль:** Створити мінімально ресурсоємну програму на Rust для відображення рівня заряду Logitech миші в system tray на Windows, Linux та macOS.

**Критичні вимоги до продуктивності:**
- **CPU:** 0% у стані очікування
- **RAM:** Максимум 3-5 MB
- **Оновлення:** Раз на 60 секунд (конфігуровано)

## 2. ТЕХНІЧНА АРХІТЕКТУРА

### 2.1 Технологічний стек

| Компонент | Технологія | Обґрунтування |
|-----------|-----------|---------------|
| Мова програмування | **Rust** | Мінімальна RAM, нема GC, кросс-платформенність  [deepu](https://deepu.tech/memory-management-in-rust/) |
| HID доступ | **hidapi** v2.6+ | Нативна підтримка Windows/Linux/macOS  [github](https://github.com/libusb/hidapi) |
| System Tray | **tray-item** або **ksni** | Легковагі бібліотеки для tray |
| Async runtime | **tokio** (minimal) | Для таймерів без постійного polling |

### 2.2 Референсний код з Solaar

**Джерело:** https://github.com/pwr-Solaar/Solaar/blob/master/lib/logitech_receiver/hidpp20.py [forums.blurbusters](https://forums.blurbusters.com/viewtopic.php?t=11663)

**Функції для портування:**

```python
# ЛОКАЦІЯ В SOLAAR:
# lib/logitech_receiver/hidpp20.py - головний файл
# lib/logitech_receiver/device.py:393-404 - entry point

def get_battery(device, battery_feature=None):
    """
    Отримує статус батареї через HID++ 2.0
    Feature 0x1000 (BATTERY_STATUS) або
    Feature 0x1004 (UNIFIED_BATTERY) або 
    Feature 0x1001 (BATTERY_VOLTAGE)
    """
    # Пріоритет:
    # 1. UNIFIED_BATTERY (0x1004) - найточніший
    # 2. BATTERY_STATUS (0x1000) - стандартний
    # 3. BATTERY_VOLTAGE (0x1001) - резервний
    
    if battery_feature == SupportedFeature.UNIFIED_BATTERY:
        return decipher_battery_unified(...)
    elif battery_feature == SupportedFeature.BATTERY_STATUS:
        return decipher_battery_status(...)
    elif battery_feature == SupportedFeature.BATTERY_VOLTAGE:
        return decipher_battery_voltage(...)

# КЛЮЧОВА ФУНКЦІЯ ДЛЯ ПОРТУВАННЯ:
def decipher_battery_unified(response):
    """
    Feature 0x1004 function 0x00: GetStatus
    
    Response format (від Solaar):
    byte[0] = discharge_level (0-100%) 
    byte [deepu](https://deepu.tech/memory-management-in-rust/) = discharge_next_level
    byte [reddit](https://www.reddit.com/r/rust/comments/x3cfcy/minimum_rust_app_memory_usage/) = battery_status:
              0x00 = discharging
              0x01 = charging (slow)
              0x02 = charging (fast)  
              0x03 = charging (wireless)
              0x04-0x07 = error states
    """
    battery_level = response[0]  # 0-100%
    next_level = response [deepu](https://deepu.tech/memory-management-in-rust/)
    status = response [reddit](https://www.reddit.com/r/rust/comments/x3cfcy/minimum_rust_app_memory_usage/)
    
    # Визначити статус
    if status == 0x00:
        charging = False
        status_text = "discharging"
    elif status in [0x01, 0x02, 0x03]:
        charging = True
        status_text = "charging"
    else:
        charging = False  
        status_text = "error"
    
    return Battery(battery_level, next_level, status_text, None)
```

**Локація коду в Solaar для детального аналізу:**
- `lib/logitech_receiver/hidpp20.py` - рядки ~1100-1200 (функції battery)
- `lib/logitech_receiver/device.py` - рядки 393-404 (метод battery())
- `lib/logitech_receiver/notifications.py` - рядки 272-276 (обробка UNIFIED_BATTERY) [fossies](https://fossies.org/linux/Solaar/lib/logitech_receiver/notifications.py)
- `lib/logitech_receiver/hidpp20_constants.py` - константи Features

## 3. ДЕТАЛЬНА СПЕЦИФІКАЦІЯ

### 3.1 HID++ Протокол

**Константи Logitech:**
```rust
const LOGITECH_VENDOR_ID: u16 = 0x046D;

// HID++ Features
const FEATURE_ROOT: u16 = 0x0000;
const FEATURE_FEATURE_SET: u16 = 0x0001;
const FEATURE_BATTERY_STATUS: u16 = 0x1000;  
const FEATURE_BATTERY_VOLTAGE: u16 = 0x1001;
const FEATURE_UNIFIED_BATTERY: u16 = 0x1004; // Найкращий варіант

// HID++ Report IDs
const HIDPP_SHORT_REPORT: u8 = 0x10;  // 7 bytes
const HIDPP_LONG_REPORT: u8 = 0x11;   // 20 bytes
```

**Алгоритм отримання батареї:**

1. **Знайти Logitech HID-пристрій:**
   ```rust
   let api = HidApi::new()?;
   for device in api.device_list() {
       if device.vendor_id() == LOGITECH_VENDOR_ID {
           // Спробувати підключитися
       }
   }
   ```

2. **Отримати індекс Feature 0x1004:**
   ```rust
   // HID++ команда GetFeature
   let mut request = [0u8; 20];
   request = HIDPP_LONG_REPORT;  // 0x11
   request [github](https://github.com/pwr/Solaar/issues/216) = 0xFF;  // Device index (any wireless)
   request [github](https://github.com/pwr-Solaar/Solaar/blob/master/po/solaar.pot) = 0x00;  // Feature ROOT index
   request [github](https://github.com/pwr-Solaar/Solaar/issues/272) = 0x00;  // Function GetFeature
   request [github](https://github.com/fredlcore/SolarEdge_Predictive_Charging) = 0x10;  // Feature ID high byte (0x1004)
   request [github](https://github.com/andyvorld/LGSTrayBattery) = 0x04;  // Feature ID low byte
   
   device.write(&request)?;
   let mut response = [0u8; 20];
   device.read_timeout(&mut response, 1000)?;
   
   let feature_index = response [github](https://github.com/fredlcore/SolarEdge_Predictive_Charging); // Індекс feature 0x1004
   ```

3. **Викликати GetStatus (function 0x00):**
   ```rust
   request = HIDPP_LONG_REPORT;
   request [github](https://github.com/pwr/Solaar/issues/216) = 0xFF;
   request [github](https://github.com/pwr-Solaar/Solaar/blob/master/po/solaar.pot) = feature_index;  // З попереднього кроку
   request [github](https://github.com/pwr-Solaar/Solaar/issues/272) = 0x00;  // Function GetStatus
   
   device.write(&request)?;
   device.read_timeout(&mut response, 1000)?;
   
   let battery_percent = response [github](https://github.com/fredlcore/SolarEdge_Predictive_Charging);  // 0-100%
   let charging = matches!(response [productcompass](https://www.productcompass.pm/p/ai-prd-template), 0x01..=0x03);
   ```

### 3.2 Архітектура програми

```
LogiTray/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point + tray setup
│   ├── hidpp/
│   │   ├── mod.rs           # HID++ протокол
│   │   ├── device.rs        # Logitech device wrapper
│   │   └── battery.rs       # Battery features (портувати з Solaar)
│   ├── tray/
│   │   ├── mod.rs
│   │   ├── windows.rs       # Windows tray impl
│   │   ├── linux.rs         # Linux tray impl  
│   │   └── macos.rs         # macOS menu bar impl
│   └── config.rs            # Конфігурація (інтервал оновлення)
└── README.md
```

### 3.3 Структури даних

```rust
// src/hidpp/battery.rs
pub struct BatteryInfo {
    pub percentage: u8,      // 0-100
    pub charging: bool,
    pub next_level: u8,      // Попередження рівень
    pub status: BatteryStatus,
}

pub enum BatteryStatus {
    Discharging,
    ChargingSlow,
    ChargingFast,
    ChargingWireless,
    Full,
    Error,
}

// src/hidpp/device.rs  
pub struct LogitechDevice {
    hid_device: HidDevice,
    feature_cache: HashMap<u16, u8>, // Feature ID -> Index
}

impl LogitechDevice {
    pub fn get_battery(&mut self) -> Result<BatteryInfo>;
    fn get_feature_index(&mut self, feature: u16) -> Result<u8>;
    fn feature_request(&self, feature_idx: u8, function: u8, params: &[u8]) -> Result<Vec<u8>>;
}
```

### 3.4 Tray Integration

**Windows:**
```rust
// src/tray/windows.rs
use tray_item::TrayItem;

pub fn create_tray(battery: &BatteryInfo) -> TrayItem {
    let mut tray = TrayItem::new(
        &format!("{}%", battery.percentage),
        "icon-path"
    ).unwrap();
    
    tray.add_label(&format!(
        "Battery: {}% ({})",
        battery.percentage,
        if battery.charging { "Charging" } else { "Discharging" }
    )).unwrap();
    
    tray.add_menu_item("Quit", || { std::process::exit(0); }).unwrap();
    tray
}
```

**Linux (ksni):**
```rust
// src/tray/linux.rs  
use ksni;

struct LogiTrayService {
    battery: Arc<Mutex<BatteryInfo>>,
}

impl ksni::TrayService for LogiTrayService {
    fn title(&self) -> String {
        format!("{}%", self.battery.lock().unwrap().percentage)
    }
    
    fn icon_name(&self) -> String {
        let bat = self.battery.lock().unwrap();
        if bat.charging {
            "battery-charging".to_string()
        } else if bat.percentage > 80 {
            "battery-full".to_string()
        } else if bat.percentage > 20 {
            "battery-good".to_string()
        } else {
            "battery-low".to_string()
        }
    }
}
```

**macOS:**
```rust
// src/tray/macos.rs
// Використати objc для NSStatusBar
```

### 3.5 Main Loop

```rust
// src/main.rs
use tokio::time::{interval, Duration};

#[tokio::main(flavor = "current_thread")] // Мінімальний runtime
async fn main() -> Result<()> {
    // 1. Ініціалізувати HID
    let mut device = LogitechDevice::new()?;
    
    // 2. Створити tray
    let tray = Arc::new(Mutex::new(create_tray_for_platform()));
    
    // 3. Таймер кожні 60 сек
    let mut timer = interval(Duration::from_secs(60));
    
    loop {
        timer.tick().await;
        
        // Отримати батарею (швидко, <5ms)
        match device.get_battery() {
            Ok(battery) => {
                // Оновити tray
                update_tray(&tray, &battery);
            }
            Err(e) => eprintln!("Battery read error: {}", e),
        }
        
        // 99.99% часу тут в sleep (0% CPU)
    }
}
```

## 4. ОПТИМІЗАЦІЯ РЕСУРСІВ

### 4.1 Мінімізація RAM

```toml
# Cargo.toml
[profile.release]
opt-level = "z"       # Оптимізація розміру
lto = true            # Link-time optimization
codegen-units = 1
strip = true          # Видалити debug symbols
panic = "abort"       # Менший binary
```

**Очікуваний розмір:**
- Binary: 500KB - 1.5MB
- Runtime RAM: 2-4 MB (hidapi ~1MB, tray ~1MB, наш код ~1MB) [reddit](https://www.reddit.com/r/rust/comments/x3cfcy/minimum_rust_app_memory_usage/)

### 4.2 Мінімізація CPU

- **Polling interval:** 60 секунд (користувач може налаштувати 30-300 сек)
- **Async sleep:** Tokio current_thread runtime (без додаткових потоків)
- **HID читання:** <5ms на запит
- **CPU usage:** 0.0% (99.99% в sleep, 0.01% на read/update)

## 5. DELIVERABLES ДЛЯ AI АГЕНТА

### Завдання 1: Портувати функції батареї з Solaar

**Файли для аналізу:**
```bash
# Завантажити з GitHub:
https://github.com/pwr-Solaar/Solaar/blob/master/lib/logitech_receiver/hidpp20.py
https://github.com/pwr-Solaar/Solaar/blob/master/lib/logitech_receiver/device.py
https://github.com/pwr-Solaar/Solaar/blob/master/lib/logitech_receiver/hidpp20_constants.py
```

**Конкретні функції для портування на Rust:**

1. **`get_battery(device)` (device.py:393-404)** [fossies](https://fossies.org/linux/Solaar/lib/logitech_receiver/device.py)
2. **`decipher_battery_unified(response)` (hidpp20.py)** [fossies](https://fossies.org/linux/Solaar/lib/logitech_receiver/notifications.py)
3. **`decipher_battery_status(response)` (hidpp20.py)** [fossies](https://fossies.org/linux/Solaar/lib/logitech_receiver/notifications.py)
4. **Feature detection logic (FeaturesArray class)** [forums.blurbusters](https://forums.blurbusters.com/viewtopic.php?t=11663)

**Output:** `src/hidpp/battery.rs` - Rust модуль з функціями отримання батареї

### Завдання 2: HID++ Communication Layer

**Input:** Специфікація HID++ з розділу 3.1

**Output:** `src/hidpp/device.rs` - struct LogitechDevice з методами:
- `new() -> Result<Self>` - знайти і підключитися до Logitech миші
- `get_feature_index(feature_id: u16) -> Result<u8>`
- `feature_request(feature_idx: u8, function: u8, params: &[u8]) -> Result<Vec<u8>>`
- `get_battery() -> Result<BatteryInfo>` - головний метод

### Завдання 3: Cross-platform Tray

**Output:** 
- `src/tray/windows.rs` - Windows tray через `tray-item`
- `src/tray/linux.rs` - Linux tray через `ksni`
- `src/tray/macos.rs` - macOS menu bar

**Функціонал:**
- Показати текст: `"85%"` або `"85% ⚡"` (якщо заряджається)
- Іконка: системна батарея (різні рівні)
- Меню: "Quit"

### Завдання 4: Main Application

**Output:** `src/main.rs`

**Алгоритм:**
1. Ініціалізувати LogitechDevice
2. Створити tray для поточної ОС
3. Запустити таймер 60 сек
4. Кожну ітерацію:
   - Прочитати батарею
   - Оновити tray
   - Sleep до наступного таймера

### Завдання 5: Cargo.toml

```toml
[package]
name = "logitray"
version = "0.1.0"
edition = "2021"

[dependencies]
hidapi = "2.6"
tokio = { version = "1", features = ["rt", "time", "macros"], default-features = false }

[target.'cfg(target_os = "windows")'.dependencies]
tray-item = "0.9"

[target.'cfg(target_os = "linux")'.dependencies]
ksni = "0.2"

[target.'cfg(target_os = "macos")'.dependencies]
cocoa = "0.25"
objc = "0.2"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

## 6. КРИТЕРІЇ УСПІХУ

✅ **Функціональні:**
- Програма коректно показує % батареї Logitech миші
- Працює на Windows 10+, Linux (Ubuntu 20.04+, Arch), macOS 12+
- Автоматично знаходить Logitech пристрої (VID 0x046D)
- є конфіг програми з настройками настраюється в %USERPROFILE%\.config\LogiBat, там настройки по інтервалу оновлення (за замовчуванням 60 сек) и от скольки процентов цифры в трее будут красными, желтыми и зелеными, наприклад 20% красные, 30% желтые, остальное зеленые выключить какой то из цветов можно написав 0%

✅ **Продуктивність:**
- RAM: ≤5 MB (виміряно через `htop` PSS на Linux) [reddit](https://www.reddit.com/r/rust/comments/x3cfcy/minimum_rust_app_memory_usage/)
- CPU: 0% у idle (99%+ часу в sleep)
- Binary: ≤2 MB після `strip`

✅ **Надійність:**
- Обробка відключення миші (graceful error handling)
- Автоматичне перепідключення при появі пристрою
- Логування помилок у stdout

## 7. РЕФЕРЕНСИ

- **Solaar repository:** https://github.com/pwr-Solaar/Solaar [github](https://github.com/pwr-Solaar/Solaar/blob/master/lib/logitech_receiver/hidpp20.py)
- **HID++ 2.0 spec:** https://lekensteyn.nl/files/logitech/logitech_hidpp_2.0_specification_draft_2012-06-04.pdf [support.logi](https://support.logi.com/hc/en-001/articles/360023359733-Mouse-battery-status-indicators)
- **hidapi docs:** https://docs.rs/hidapi [docs](https://docs.rs/hidapi)
- **Приклад elem (Rust tray):** https://github.com/Fuwn/elem [github](https://github.com/Fuwn/elem)

***

**Інструкції для AI агента (Claude Code, Cursor):**

1. Завантажте файли з Solaar (посилання вище)
2. Проаналізуйте функції `get_battery`, `decipher_battery_unified` з Python коду, вони вже скачані C:\Projectrs\batrust\Solaar
3. Створіть Rust проєкт згідно структури з розділу 3.2
4. Портуйте логіку батареї з Python на Rust (розділ 3.3)
5. Реалізуйте HID++ комунікацію (розділ 3.1)
6. Додайте platform-specific tray код (розділ 3.4)
7. Зберіть main loop (розділ 3.5)
8. Оптимізуйте згідно розділу 4

**Пріоритет:** Мінімальне споживання ресурсів > Швидкість розробки > Додатковий функціонал