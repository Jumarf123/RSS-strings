use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    English,
    Russian,
    Italian,
    Espanol,
}

pub const OPTIONS_LABEL: &str = "Options";
pub const LANGUAGES_LABEL: &str = "Languages";
pub const LANGUAGE_ORDER: [Language; 4] = [
    Language::Italian,
    Language::Russian,
    Language::English,
    Language::Espanol,
];

impl Language {
    pub fn display_name(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Russian => "Russian",
            Language::Italian => "Italian",
            Language::Espanol => "Espanol",
        }
    }

    pub fn from_system_or_english() -> Self {
        Self::from_system().unwrap_or(Language::English)
    }

    pub fn from_system() -> Option<Self> {
        system_locale().and_then(|locale| Self::from_locale(&locale))
    }

    pub fn from_locale(locale: &str) -> Option<Self> {
        let lang = locale
            .split(|c| c == '-' || c == '_')
            .next()
            .unwrap_or(locale)
            .to_ascii_lowercase();
        match lang.as_str() {
            "en" => Some(Language::English),
            "ru" => Some(Language::Russian),
            "it" => Some(Language::Italian),
            "es" => Some(Language::Espanol),
            _ => None,
        }
    }
}

fn system_locale() -> Option<String> {
    #[cfg(windows)]
    {
        use windows::Win32::Globalization::GetUserDefaultLocaleName;
        const LOCALE_NAME_MAX: usize = 85;
        let mut buffer = [0u16; LOCALE_NAME_MAX];
        let len = unsafe { GetUserDefaultLocaleName(&mut buffer) };
        if len <= 0 {
            return None;
        }
        let len = (len as usize).saturating_sub(1);
        Some(String::from_utf16_lossy(&buffer[..len]))
    }
    #[cfg(not(windows))]
    {
        std::env::var("LANG").ok()
    }
}

#[derive(Clone, Copy)]
pub struct UiText {
    lang: Language,
}

impl UiText {
    pub fn new(lang: Language) -> Self {
        Self { lang }
    }

    pub fn status_ready(&self) -> &'static str {
        match self.lang {
            Language::English => "Ready",
            Language::Russian => "Готов",
            Language::Italian => "Pronto",
            Language::Espanol => "Listo",
        }
    }

    pub fn status_select_process(&self) -> &'static str {
        match self.lang {
            Language::English => "Select a process",
            Language::Russian => "Выберите процесс",
            Language::Italian => "Seleziona un processo",
            Language::Espanol => "Selecciona un proceso",
        }
    }

    pub fn status_add_strings(&self) -> &'static str {
        match self.lang {
            Language::English => "Add strings to search",
            Language::Russian => "Добавьте строки для поиска",
            Language::Italian => "Aggiungi stringhe da cercare",
            Language::Espanol => "Agrega cadenas para buscar",
        }
    }

    pub fn status_scanning(&self) -> &'static str {
        match self.lang {
            Language::English => "Scanning…",
            Language::Russian => "Сканирование…",
            Language::Italian => "Scansione…",
            Language::Espanol => "Escaneando…",
        }
    }

    pub fn status_start_error(&self) -> &'static str {
        match self.lang {
            Language::English => "Start error",
            Language::Russian => "Ошибка запуска",
            Language::Italian => "Errore di avvio",
            Language::Espanol => "Error de inicio",
        }
    }

    pub fn status_process_gone(&self) -> &'static str {
        match self.lang {
            Language::English => "Process disappeared",
            Language::Russian => "Процесс исчез",
            Language::Italian => "Il processo è scomparso",
            Language::Espanol => "El proceso desapareció",
        }
    }

    pub fn status_stopped(&self) -> &'static str {
        match self.lang {
            Language::English => "Stopped",
            Language::Russian => "Остановлено",
            Language::Italian => "Interrotto",
            Language::Espanol => "Detenido",
        }
    }

    pub fn status_done(&self) -> &'static str {
        match self.lang {
            Language::English => "Done",
            Language::Russian => "Готово",
            Language::Italian => "Completato",
            Language::Espanol => "Completado",
        }
    }

    pub fn status_selected_pid(&self, pid: u32) -> String {
        match self.lang {
            Language::English => format!("Selected PID {pid}"),
            Language::Russian => format!("Выбран PID {pid}"),
            Language::Italian => format!("Selezionato PID {pid}"),
            Language::Espanol => format!("PID seleccionado {pid}"),
        }
    }

    pub fn status_line(&self, status: &str, found: usize) -> String {
        match self.lang {
            Language::English => format!("Status: {status} | Found: {found}"),
            Language::Russian => format!("Статус: {status} | Найдено: {found}"),
            Language::Italian => format!("Stato: {status} | Trovate: {found}"),
            Language::Espanol => format!("Estado: {status} | Encontradas: {found}"),
        }
    }

    pub fn progress_line(&self, processed: usize, total: usize, matches: usize) -> String {
        match self.lang {
            Language::English => format!("Regions: {processed}/{total} | Matches: {matches}"),
            Language::Russian => format!("Регионов: {processed}/{total} | Совпадений: {matches}"),
            Language::Italian => format!("Regioni: {processed}/{total} | Corrispondenze: {matches}"),
            Language::Espanol => format!("Regiones: {processed}/{total} | Coincidencias: {matches}"),
        }
    }

    pub fn button_refresh(&self) -> &'static str {
        match self.lang {
            Language::English => "Refresh",
            Language::Russian => "Обновить",
            Language::Italian => "Aggiorna",
            Language::Espanol => "Actualizar",
        }
    }

    pub fn button_find(&self) -> &'static str {
        match self.lang {
            Language::English => "Find",
            Language::Russian => "Найти",
            Language::Italian => "Trova",
            Language::Espanol => "Buscar",
        }
    }

    pub fn button_scan(&self) -> &'static str {
        match self.lang {
            Language::English => "Scan",
            Language::Russian => "Сканировать",
            Language::Italian => "Scansiona",
            Language::Espanol => "Escanear",
        }
    }

    pub fn button_stop(&self) -> &'static str {
        match self.lang {
            Language::English => "Stop",
            Language::Russian => "Стоп",
            Language::Italian => "Ferma",
            Language::Espanol => "Detener",
        }
    }

    pub fn label_searching(&self) -> &'static str {
        match self.lang {
            Language::English => "Searching strings…",
            Language::Russian => "Идёт поиск строк…",
            Language::Italian => "Ricerca stringhe…",
            Language::Espanol => "Buscando cadenas…",
        }
    }

    pub fn button_cancel(&self) -> &'static str {
        match self.lang {
            Language::English => "Cancel",
            Language::Russian => "Отмена",
            Language::Italian => "Annulla",
            Language::Espanol => "Cancelar",
        }
    }

    pub fn error_with(&self, err: &str) -> String {
        match self.lang {
            Language::English => format!("Error: {err}"),
            Language::Russian => format!("Ошибка: {err}"),
            Language::Italian => format!("Errore: {err}"),
            Language::Espanol => format!("Error: {err}"),
        }
    }

    pub fn button_hide(&self) -> &'static str {
        match self.lang {
            Language::English => "Hide",
            Language::Russian => "Скрыть",
            Language::Italian => "Nascondi",
            Language::Espanol => "Ocultar",
        }
    }

    pub fn header_search_settings(&self) -> &'static str {
        match self.lang {
            Language::English => "Search settings",
            Language::Russian => "Настройки поиска",
            Language::Italian => "Impostazioni di ricerca",
            Language::Espanol => "Configuración de búsqueda",
        }
    }

    pub fn label_min_length(&self) -> &'static str {
        match self.lang {
            Language::English => "Minimum length:",
            Language::Russian => "Минимальная длина:",
            Language::Italian => "Lunghezza minima:",
            Language::Espanol => "Longitud mínima:",
        }
    }

    pub fn label_detect_unicode(&self) -> &'static str {
        match self.lang {
            Language::English => "Detect Unicode (UTF-16 LE)",
            Language::Russian => "Определять Unicode (UTF-16 LE)",
            Language::Italian => "Rileva Unicode (UTF-16 LE)",
            Language::Espanol => "Detectar Unicode (UTF-16 LE)",
        }
    }

    pub fn label_extended_unicode(&self) -> &'static str {
        match self.lang {
            Language::English => "Extended Unicode",
            Language::Russian => "Расширенный Unicode",
            Language::Italian => "Unicode esteso",
            Language::Espanol => "Unicode extendido",
        }
    }

    pub fn label_private(&self) -> &'static str {
        match self.lang {
            Language::English => "Private",
            Language::Russian => "Частные",
            Language::Italian => "Privato",
            Language::Espanol => "Privado",
        }
    }

    pub fn label_image(&self) -> &'static str {
        match self.lang {
            Language::English => "Image",
            Language::Russian => "Образ",
            Language::Italian => "Immagine",
            Language::Espanol => "Imagen",
        }
    }

    pub fn label_mapped(&self) -> &'static str {
        match self.lang {
            Language::English => "Mapped",
            Language::Russian => "Отображённые",
            Language::Italian => "Mappato",
            Language::Espanol => "Mapeado",
        }
    }

    pub fn label_enable_debug_privilege(&self) -> &'static str {
        match self.lang {
            Language::English => "Enable SeDebugPrivilege (requires restarting some processes)",
            Language::Russian => "Включить SeDebugPrivilege (требует перезапуска некоторых процессов)",
            Language::Italian => "Abilita SeDebugPrivilege (richiede il riavvio di alcuni processi)",
            Language::Espanol => "Habilitar SeDebugPrivilege (requiere reiniciar algunos procesos)",
        }
    }

    pub fn label_settings_saved(&self) -> &'static str {
        match self.lang {
            Language::English => "Settings are saved automatically",
            Language::Russian => "Настройки сохраняются автоматически",
            Language::Italian => "Le impostazioni vengono salvate automaticamente",
            Language::Espanol => "La configuración se guarda automáticamente",
        }
    }

    pub fn header_strings_input(&self) -> &'static str {
        match self.lang {
            Language::English => "Strings to search",
            Language::Russian => "Строки для поиска",
            Language::Italian => "Stringhe da cercare",
            Language::Espanol => "Cadenas para buscar",
        }
    }

    pub fn label_lines_count(&self, count: usize, limit: usize) -> String {
        match self.lang {
            Language::English => format!("Lines: {count} / {limit}"),
            Language::Russian => format!("Строк: {count} / {limit}"),
            Language::Italian => format!("Righe: {count} / {limit}"),
            Language::Espanol => format!("Líneas: {count} / {limit}"),
        }
    }

    pub fn label_no_lines(&self) -> &'static str {
        match self.lang {
            Language::English => "No lines — paste text on the left",
            Language::Russian => "Нет строк — вставьте текст слева",
            Language::Italian => "Nessuna riga — incolla il testo a sinistra",
            Language::Espanol => "No hay líneas — pega el texto a la izquierda",
        }
    }

    pub fn label_trimmed(&self, limit: usize) -> String {
        match self.lang {
            Language::English => format!("Trimmed to {limit} lines"),
            Language::Russian => format!("Обрезано до {limit} строк"),
            Language::Italian => format!("Limitato a {limit} righe"),
            Language::Espanol => format!("Limitado a {limit} líneas"),
        }
    }

    pub fn header_results(&self) -> &'static str {
        match self.lang {
            Language::English => "Results",
            Language::Russian => "Результаты",
            Language::Italian => "Risultati",
            Language::Espanol => "Resultados",
        }
    }

    pub fn label_no_results_prompt(&self) -> &'static str {
        match self.lang {
            Language::English => "No results. Click “Scan”.",
            Language::Russian => "Результатов нет. Нажмите «Сканировать».",
            Language::Italian => "Nessun risultato. Premi «Scansiona».",
            Language::Espanol => "Sin resultados. Haz clic en «Escanear».",
        }
    }

    pub fn label_no_results_yet(&self) -> &'static str {
        match self.lang {
            Language::English => "No results yet",
            Language::Russian => "Результатов пока нет",
            Language::Italian => "Nessun risultato per ora",
            Language::Espanol => "Aún no hay resultados",
        }
    }

    pub fn process_label(&self) -> &'static str {
        match self.lang {
            Language::English => "Process:",
            Language::Russian => "Процесс:",
            Language::Italian => "Processo:",
            Language::Espanol => "Proceso:",
        }
    }

    pub fn process_filter_hint(&self) -> &'static str {
        match self.lang {
            Language::English => "PID or name fragment",
            Language::Russian => "PID или фрагмент имени",
            Language::Italian => "PID o parte del nome",
            Language::Espanol => "PID o parte del nombre",
        }
    }

    pub fn process_not_selected(&self) -> &'static str {
        match self.lang {
            Language::English => "Not selected",
            Language::Russian => "Не выбран",
            Language::Italian => "Non selezionato",
            Language::Espanol => "No seleccionado",
        }
    }

    pub fn process_limited_suffix(&self) -> &'static str {
        match self.lang {
            Language::English => " • limited",
            Language::Russian => " • ограничен",
            Language::Italian => " • limitato",
            Language::Espanol => " • limitado",
        }
    }

    pub fn strings_input_label(&self) -> &'static str {
        match self.lang {
            Language::English => "Paste strings to search (one per line):",
            Language::Russian => "Вставьте строки для поиска (по одной на строке):",
            Language::Italian => "Incolla le stringhe da cercare (una per riga):",
            Language::Espanol => "Pega las cadenas para buscar (una por línea):",
        }
    }

    pub fn button_clear(&self) -> &'static str {
        match self.lang {
            Language::English => "Clear",
            Language::Russian => "Очистить",
            Language::Italian => "Cancella",
            Language::Espanol => "Limpiar",
        }
    }

    pub fn button_insert_sample(&self) -> &'static str {
        match self.lang {
            Language::English => "Insert sample",
            Language::Russian => "Вставить пример",
            Language::Italian => "Inserisci esempio",
            Language::Espanol => "Insertar ejemplo",
        }
    }

    pub fn button_load_file(&self) -> &'static str {
        match self.lang {
            Language::English => "Load from file",
            Language::Russian => "Загрузить из файла",
            Language::Italian => "Carica da file",
            Language::Espanol => "Cargar desde archivo",
        }
    }

    pub fn file_filter_text(&self) -> &'static str {
        match self.lang {
            Language::English => "Text",
            Language::Russian => "Текст",
            Language::Italian => "Testo",
            Language::Espanol => "Texto",
        }
    }

    pub fn strings_hint(&self, limit: usize) -> String {
        match self.lang {
            Language::English => format!("One line per entry, up to {limit} lines"),
            Language::Russian => format!("Одна строка на строку, до {limit} строк"),
            Language::Italian => format!("Una riga per voce, fino a {limit} righe"),
            Language::Espanol => format!("Una línea por entrada, hasta {limit} líneas"),
        }
    }

    pub fn results_filter_label(&self) -> &'static str {
        match self.lang {
            Language::English => "Filter results:",
            Language::Russian => "Фильтр по результатам:",
            Language::Italian => "Filtra risultati:",
            Language::Espanol => "Filtrar resultados:",
        }
    }

    pub fn results_filter_hint(&self) -> &'static str {
        match self.lang {
            Language::English => "substring for Query/Match",
            Language::Russian => "подстрока для Query/Match",
            Language::Italian => "sottostringa per Query/Match",
            Language::Espanol => "subcadena para Query/Match",
        }
    }

    pub fn button_copy_all(&self) -> &'static str {
        match self.lang {
            Language::English => "📋 Copy all",
            Language::Russian => "📋 Копировать все",
            Language::Italian => "📋 Copia tutto",
            Language::Espanol => "📋 Copiar todo",
        }
    }

    pub fn button_save_all(&self) -> &'static str {
        match self.lang {
            Language::English => "💾 Save all",
            Language::Russian => "💾 Сохранить все",
            Language::Italian => "💾 Salva tutto",
            Language::Espanol => "💾 Guardar todo",
        }
    }

    pub fn button_prev_page(&self) -> &'static str {
        match self.lang {
            Language::English => "Prev",
            Language::Russian => "Назад",
            Language::Italian => "Indietro",
            Language::Espanol => "Atrás",
        }
    }

    pub fn button_next_page(&self) -> &'static str {
        match self.lang {
            Language::English => "Next",
            Language::Russian => "Вперёд",
            Language::Italian => "Avanti",
            Language::Espanol => "Siguiente",
        }
    }

    pub fn label_page(&self, page: usize, total: usize) -> String {
        match self.lang {
            Language::English => format!("Page {page}/{total}"),
            Language::Russian => format!("Страница {page}/{total}"),
            Language::Italian => format!("Pagina {page}/{total}"),
            Language::Espanol => format!("Página {page}/{total}"),
        }
    }

    pub fn column_query(&self) -> &'static str {
        match self.lang {
            Language::English => "Requested string",
            Language::Russian => "Запрошенная строка",
            Language::Italian => "Stringa richiesta",
            Language::Espanol => "Cadena solicitada",
        }
    }

    pub fn column_match(&self) -> &'static str {
        match self.lang {
            Language::English => "Match in process",
            Language::Russian => "Совпадение в процессе",
            Language::Italian => "Corrispondenza nel processo",
            Language::Espanol => "Coincidencia en el proceso",
        }
    }

    pub fn column_address(&self) -> &'static str {
        match self.lang {
            Language::English => "Address",
            Language::Russian => "Адрес",
            Language::Italian => "Indirizzo",
            Language::Espanol => "Dirección",
        }
    }

    pub fn column_region(&self) -> &'static str {
        match self.lang {
            Language::English => "Region",
            Language::Russian => "Регион",
            Language::Italian => "Regione",
            Language::Espanol => "Región",
        }
    }

    pub fn column_encoding(&self) -> &'static str {
        match self.lang {
            Language::English => "Encoding",
            Language::Russian => "Кодировка",
            Language::Italian => "Codifica",
            Language::Espanol => "Codificación",
        }
    }

    pub fn tooltip_copy_addresses(&self) -> &'static str {
        match self.lang {
            Language::English => "Copy addresses",
            Language::Russian => "Копировать адреса",
            Language::Italian => "Copia indirizzi",
            Language::Espanol => "Copiar direcciones",
        }
    }

    pub fn context_copy_cell(&self) -> &'static str {
        match self.lang {
            Language::English => "Copy cell",
            Language::Russian => "Копировать ячейку",
            Language::Italian => "Copia cella",
            Language::Espanol => "Copiar celda",
        }
    }

    pub fn context_copy_row(&self) -> &'static str {
        match self.lang {
            Language::English => "Copy full row",
            Language::Russian => "Копировать всю строку",
            Language::Italian => "Copia riga completa",
            Language::Espanol => "Copiar fila completa",
        }
    }

    pub fn action_copy_column(&self) -> &'static str {
        match self.lang {
            Language::English => "Copy column (filter/selection)",
            Language::Russian => "Копировать колонку (фильтр/выделение)",
            Language::Italian => "Copia colonna (filtro/selezione)",
            Language::Espanol => "Copiar columna (filtro/selección)",
        }
    }

    pub fn action_save_column(&self) -> &'static str {
        match self.lang {
            Language::English => "Save column (filter/selection)",
            Language::Russian => "Сохранить колонку (фильтр/выделение)",
            Language::Italian => "Salva colonna (filtro/selezione)",
            Language::Espanol => "Guardar columna (filtro/selección)",
        }
    }

    pub fn region_private(&self) -> &'static str {
        match self.lang {
            Language::English => "Private",
            Language::Russian => "Частный",
            Language::Italian => "Privato",
            Language::Espanol => "Privado",
        }
    }

    pub fn region_image(&self) -> &'static str {
        match self.lang {
            Language::English => "Image",
            Language::Russian => "Образ",
            Language::Italian => "Immagine",
            Language::Espanol => "Imagen",
        }
    }

    pub fn region_mapped(&self) -> &'static str {
        match self.lang {
            Language::English => "Mapped",
            Language::Russian => "Отображённый",
            Language::Italian => "Mappato",
            Language::Espanol => "Mapeado",
        }
    }

    pub fn region_other(&self) -> &'static str {
        match self.lang {
            Language::English => "Other",
            Language::Russian => "Другое",
            Language::Italian => "Altro",
            Language::Espanol => "Otro",
        }
    }

    pub fn admin_title(&self) -> &'static str {
        match self.lang {
            Language::English => "Administrator privileges required",
            Language::Russian => "Требуются права администратора",
            Language::Italian => "Privilegi di amministratore richiesti",
            Language::Espanol => "Se requieren privilegios de administrador",
        }
    }

    pub fn admin_body(&self) -> &'static str {
        match self.lang {
            Language::English => "RSS-strings must be run as administrator.\nRestart the application with elevated rights.",
            Language::Russian => "RSS-strings должен быть запущен от имени администратора.\nПерезапустите приложение с повышенными правами.",
            Language::Italian => "RSS-strings deve essere eseguito come amministratore.\nRiavvia l'applicazione con diritti elevati.",
            Language::Espanol => "RSS-strings debe ejecutarse como administrador.\nReinicia la aplicación con privilegios elevados.",
        }
    }

    pub fn error_empty_list(&self) -> &'static str {
        match self.lang {
            Language::English => "The string list is empty",
            Language::Russian => "Список строк пуст",
            Language::Italian => "L'elenco delle stringhe è vuoto",
            Language::Espanol => "La lista de cadenas está vacía",
        }
    }

    pub fn error_no_valid_queries(&self) -> &'static str {
        match self.lang {
            Language::English => "No valid strings to search",
            Language::Russian => "Нет валидных строк для поиска",
            Language::Italian => "Nessuna stringa valida da cercare",
            Language::Espanol => "No hay cadenas válidas para buscar",
        }
    }

    pub fn error_open_process(&self, pid: u32) -> String {
        match self.lang {
            Language::English => format!("Failed to open process {pid}"),
            Language::Russian => format!("Не удалось открыть процесс {pid}"),
            Language::Italian => format!("Impossibile aprire il processo {pid}"),
            Language::Espanol => format!("No se pudo abrir el proceso {pid}"),
        }
    }

    pub fn error_regions(&self) -> &'static str {
        match self.lang {
            Language::English => "Failed to enumerate memory regions",
            Language::Russian => "Ошибка перечисления регионов памяти",
            Language::Italian => "Impossibile enumerare le regioni di memoria",
            Language::Espanol => "No se pudieron enumerar las regiones de memoria",
        }
    }

    pub fn error_matcher_build(&self) -> &'static str {
        match self.lang {
            Language::English => "Failed to build Aho-Corasick matcher",
            Language::Russian => "Не удалось построить автомат Aho-Corasick",
            Language::Italian => "Impossibile creare l'automa Aho-Corasick",
            Language::Espanol => "No se pudo crear el autómata Aho-Corasick",
        }
    }
}
