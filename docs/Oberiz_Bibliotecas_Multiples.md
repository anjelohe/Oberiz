# Bibliotecas en varios discos — desarrollo del 14 de octubre de 2026

Incluido en v1.0.0. La publicación no actualiza automáticamente la instancia instalada del usuario. Para conexiones SMB de Windows, véase [Conexiones SMB](Oberiz_SMB.md).

## Configuración

En Settings → Library folders hay listas independientes de películas y series. Cada entrada permite escribir o explorar una carpeta, añadir otra y quitarla. «Cargar carpetas de categorías» consulta qBittorrent; seleccionar una categoría copia su ruta a la lista, aplicando los mapeos remotos configurados. No modifica la categoría ni su destino en qBittorrent.

Las carpetas deben ser accesibles desde el servidor de Oberiz, no necesariamente desde el navegador. En Docker deben estar montadas; con qBittorrent remoto, configurar las correspondencias remoto → local. Los destinos de nuevas importaciones siguen siendo campos separados y únicos. Cambiar las carpetas de reconocimiento no mueve, renombra ni borra contenido.

Las rutas Movies/Series anteriores se conservan como raíces iniciales si todavía no se ha guardado una lista. Una lista vacía explícita no recupera la ruta antigua. Se eliminan entradas repetidas y se recorre una sola vez cada árbol solapado.

## Escaneo y descargas

- Escaneo manual y periódico configurable: 120 minutos inicialmente, desactivado hasta que se habilite y guarde.
- Antes de la primera búsqueda automática de un título se escanean las raíces configuradas. RSS y automatización comprueban también torrents equivalentes en qBittorrent.
- Los torrents externos completados se reconocen por sus rutas de contenido accesibles, aunque otros capítulos de esa serie ya estén indexados. Las consultas de esos torrents se reutilizan; sus rutas se revisan periódicamente o cuando se actualizan los metadatos.
- Un torrent pendiente o pausado impide obtener una segunda copia del mismo episodio/película; no se reanuda automáticamente. Un torrent completado que no sea accesible protege igualmente frente a duplicados, sin inventar que su archivo está indexado.
- Si un disco está desconectado, da error de lectura o aparece vacío tras haber contenido archivos registrados, se conserva su disponibilidad anterior. Se detiene la adquisición automática del tipo de contenido afectado hasta recuperar la ruta y realizar un escaneo correcto.
- No se borran archivos ni asociaciones al quitar una carpeta de configuración. Las asociaciones conocidas se conservan al volver a escanear.
- Las mejoras siguen sujetas a Allow upgrades, el cutoff y la calidad ya instalada. No se desactiva la monitorización al terminar una descarga.

## Series y estadísticas

Cada temporada distingue faltante, disponible parcialmente, al día y completa. La temporada más reciente de una serie en emisión no se considera completa solo porque estén todos los capítulos conocidos. Un estado desconocido se trata de forma conservadora.

«Seguir próximos episodios» es un control por temporada, independiente de seguir futuras temporadas. Guarda la fecha de inicio: los episodios que se emitan después siguen monitorizados cuando ya hayan pasado su fecha de emisión. Respeta las exclusiones manuales posteriores. No recupera los capítulos anteriores a esa fecha.

El Dashboard añade series monitorizadas y episodios disponibles, emitidos pendientes y futuros, agrupados en una tarjeta con dos columnas. La fila de tarjetas se adapta al espacio disponible. Almacenamiento indexado suma los archivos reconocidos, no la capacidad de los discos ni todos los torrents; configurar y escanear queda exclusivamente en Settings.

El explorador de carpetas y el listado de categorías admiten selección múltiple. En Windows se conserva correctamente el prefijo de red `\\servidor\recurso`; los borradores `UNC\servidor\recurso` emitidos por el explorador anterior se reparan al guardar. Las correspondencias completamente vacías se ignoran, no bloquean el guardado.

## Integraciones

La API pública devuelve `availability_status` y `seasons` con contadores y estados por temporada. Para seguir próximos episodios de una temporada ya parcialmente disponible: `requested_seasons: [2]`, `monitor_mode: "future"`, `monitor_future_seasons: false`. Los clientes externos deben utilizar estos datos para no deshabilitar una temporada en emisión por tener algunos episodios disponibles. La interfaz del portal externo no está en este repositorio y no se ha modificado aquí.

Rutas internas añadidas: `GET /api/library/roots`. Settings admite `movie_library_paths`, `series_library_paths`, `qbittorrent_path_mappings`, `library_scan_enabled` y `library_scan_interval_minutes`. Los estados de raíces incluyen accesibilidad, último escaneo y error.

## Verificación

Pruebas con bases de datos y carpetas temporales: varios discos, solapamiento, desconexión, montaje vacío, detección de episodios externos con otra copia ya indexada, estados de torrents preservados y seguimiento futuro persistente. La migración conserva los datos de instalaciones previas. Compilación de frontend, cobertura de seis idiomas y análisis estático de backend también verificados.

Copia previa: `C:\Users\anjelohe\Desktop\Oberiz-copia-20261014-053632.zip`. Incluye código, configuración, historial Git y snapshots SQLite coherentes. Contiene datos privados: no publicar este ZIP.
