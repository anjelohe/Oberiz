// Interface copy only: media titles, paths, tracker definitions and protocol values are untouched.
export const spanish: Record<string, string> = Object.fromEntries(String.raw`
Search cycles per day|Ciclos de búsqueda al día
Keep schedule after restart|Conservar la programación tras reiniciar
Daily cycles use random times. Missed cycles are not replayed. Manual and on-add searches are additional.|Los ciclos diarios usan horarios aleatorios. Los ciclos perdidos no se recuperan. Las búsquedas manuales y al añadir contenido son adicionales.
Missing content reports|Informes de contenido faltante
Metadata refresh interval (days)|Renovación de metadatos (días)
Finished series refresh interval (days)|Revisión de series finalizadas (días)
Inventory check interval (hours)|Comprobación del inventario (horas)
Full inventory reconciliation (days)|Reconciliación completa del inventario (días)
Intervals apply when checking reports or synchronizing Cinetta. Cached metadata is reused; finished series are checked less often. Updated Cinetta sender required for incremental inventory.|Los intervalos se aplican al comprobar informes o sincronizar Cinetta. Se reutilizan los metadatos guardados y las series finalizadas se revisan con menor frecuencia. El inventario incremental requiere actualizar el emisor de Cinetta.
Reports use the last inventory Cinetta sent to Oberiz. Updating a report does not request a new sync or require another API key.|Los informes usan el último inventario que Cinetta ha enviado a Oberiz. Actualizar un informe no solicita una nueva sincronización ni requiere otra clave API.
No Cinetta inventory received yet. Enable inventory sending in Cinetta using the existing Oberiz API key.|Todavía no se ha recibido el inventario de Cinetta. Activa su envío en Cinetta usando la clave API de Oberiz que ya tienes.
Read-only reports from Cinetta, separate from the managed library.|Informes de solo lectura desde Cinetta, separados de la biblioteca gestionada.
Cinetta URL|Dirección de Cinetta
Cinetta inventory read key|Clave de lectura del inventario de Cinetta
Create a library:read key in Cinetta and enable inventory reading there. This is independent of inventory reporting to Oberiz.|Crea una clave library:read en Cinetta y permite allí la lectura del inventario. Es independiente del envío de inventario a Oberiz.
Allow integrations to read and refresh missing reports|Permitir a las integraciones consultar y actualizar informes de faltantes
Disabled by default. Uses the existing Oberiz API key; local report buttons do not need this external permission.|Desactivado por defecto. Usa la clave API de Oberiz; los botones locales no necesitan este permiso externo.
Check missing content|Comprobar faltantes
Check collection|Comprobar colección
Incomplete collections|Colecciones incompletas
Read-only report. No requests, monitoring or downloads are created.|Informe de solo lectura. No crea peticiones, monitorización ni descargas.
Scope|Alcance
One series|Una serie concreta
One collection|Una colección concreta
Entire library|Toda la biblioteca
Last saved report|Último informe guardado
Stop waiting|Dejar de esperar
Updating will check the entire inventory and may take several minutes.|Actualizar comprobará todo el inventario y puede tardar varios minutos.
Checking inventory and release metadata…|Comprobando inventario y fechas de estreno…
Provisional report|Informe provisional
No missing content found|No se ha encontrado contenido faltante
Report complete|Informe completo
Report date|Fecha del informe
Inventory date|Fecha del inventario
Not listed in Cinetta. Missing files are not physically confirmed; offline indexed items are not marked missing.|No consta en Cinetta. La ausencia física no está confirmada; los elementos indexados en discos desconectados no se consideran faltantes.
Groups|Grupos
Warnings|Advertencias
Configure Cinetta inventory access in Settings first.|Configura primero el acceso al inventario de Cinetta en Ajustes.
No saved report. Use Update to start a check.|No hay informe guardado. Pulsa Actualizar para comprobarlo.
Cinetta rejected the inventory key.|Cinetta ha rechazado la clave de inventario.
Cinetta does not support inventory queries.|Cinetta no admite consultas de inventario.
Cinetta is unavailable.|Cinetta no está disponible.
This movie does not belong to a TMDB collection.|Esta película no pertenece a una colección de TMDB.
Choose a valid series or collection ID.|Elige un identificador válido de serie o colección.
Other checks are running. Try again later.|Hay otras comprobaciones en curso. Inténtalo más tarde.
Could not check missing content.|No se pudo comprobar el contenido faltante.
The check is still running. Consult the saved report later.|La comprobación sigue en curso. Consulta el informe guardado más tarde.
The check failed. The previous report is unchanged.|La comprobación ha fallado. Se conserva el informe anterior.
Dashboard|Inicio
Library inventory|Inventario de la biblioteca
Pause new acquisitions if Cinetta inventory is missing or older than 24 hours|Pausar nuevas adquisiciones si no hay inventario de Cinetta o tiene más de 24 horas
qBittorrent does not provide atomic no-overwrite renaming. Visible destinations are checked; do not run other organizers on these folders simultaneously.|qBittorrent no ofrece un renombrado atómico sin reemplazo. Se comprueban los destinos visibles; no ejecutes otros organizadores sobre estas carpetas al mismo tiempo.
Choose one source. Download destinations are managed in qBittorrent categories.|Elige el origen del inventario. Los destinos de descarga se configuran en las categorías de qBittorrent.
Receive the library through the public API. No NAS access required.|Recibe la biblioteca por la API pública. No requiere acceso al NAS.
Local folders|Carpetas locales
Read-only scanning. Oberiz never imports, copies or moves media files.|Escaneo de solo lectura. Oberiz nunca importa, copia ni mueve archivos multimedia.
Enable Public API v1, generate a key and save Settings. In Cinetta, configure the Oberiz URL and key and enable inventory reporting.|Activa API pública v1, genera una clave y guarda los ajustes. En Cinetta, configura la dirección y la clave de Oberiz y activa el envío del inventario.
Cinetta sends the inventory; selecting this mode does not initiate a sync.|Cinetta envía el inventario; seleccionar este modo no inicia una sincronización.
Last inventory received:|Último inventario recibido:
Not received yet|Todavía no se ha recibido
If Cinetta stops reporting, the last inventory is preserved. Local scanning is never enabled automatically.|Si Cinetta deja de informar, se conserva el último inventario. El escaneo local nunca se activa automáticamente.
Optional renaming is configured separately and performed only through qBittorrent. Recovery metadata is included in Oberiz database backups.|El renombrado opcional se configura aparte y se ejecuta solo mediante qBittorrent. Las copias de la base de datos de Oberiz incluyen los metadatos de recuperación.
No recent Cinetta inventory: automatic acquisition is held until a report is received (maximum age: 24 hours).|No hay inventario reciente de Cinetta: las nuevas adquisiciones quedan en espera hasta recibirlo (antigüedad máxima: 24 horas).
qBittorrent naming & recovery|Renombrado mediante qBittorrent y recuperación
Disabled by default. Oberiz requests changes through qBittorrent; it never opens media files.|Desactivado por defecto. Oberiz solicita los cambios a qBittorrent; nunca abre los archivos multimedia.
Rename completed downloads through qBittorrent|Renombrar descargas completadas mediante qBittorrent
Apply sharing policies (keep all downloaded files)|Aplicar políticas de compartición (conservar todos los archivos descargados)
Only Oberiz downloads are processed. Ambiguous files, external subtitles and naming conflicts require manual review.|Solo se procesan las descargas de Oberiz. Si hay archivos ambiguos, subtítulos externos o conflictos de nombres, se detiene el renombrado para revisión.
Set download destinations in qBittorrent categories. Oberiz does not need NAS credentials.|Configura los destinos de descarga en las categorías de qBittorrent. Oberiz no necesita credenciales del NAS.
Downloads & recovery|Descargas y recuperación
Torrent metadata, destinations and naming history. Recovery never starts downloading automatically.|Metadatos del torrent, destinos e historial de nombres. La recuperación nunca inicia descargas automáticamente.
Restore stopped|Recuperar detenido
Restore this torrent stopped in qBittorrent, restore its recorded names and request a recheck? Inspect the result before resuming manually.|¿Recuperar este torrent detenido en qBittorrent, restaurar los nombres registrados y solicitar una comprobación? Revisa el resultado antes de reanudarlo manualmente.
Restored stopped in {0}. Recheck requested; inspect and resume in qBittorrent.|Recuperado detenido en {0}. Comprobación solicitada; revisa y reanuda en qBittorrent.
Downloaded files are always kept.|Los archivos descargados se conservan siempre.
SMB / CIFS servers|Servidores SMB / CIFS
Server|Servidor
Open|Abrir
Forget saved connection|Olvidar conexión guardada
{0} catalogue definitions could not be installed; the rest were updated. Details in History.|{0} definiciones del catálogo no se pudieron instalar; el resto se ha actualizado. Detalles en Historial.
No mapping needed: Oberiz already opens every folder qBittorrent uses, or no matching shared folder was found.|No hace falta ninguna correspondencia: Oberiz ya abre todas las carpetas que usa qBittorrent, o no se encontró ninguna carpeta compartida que coincida.
{0} mappings added. Save Settings to apply them.|{0} correspondencias añadidas. Guarda los ajustes para aplicarlas.
Files and storage|Archivos y almacenamiento
Where qBittorrent downloads, where your movies and series end up, and how Oberiz reaches them.|Dónde descarga qBittorrent, dónde acaban tus películas y series, y cómo llega Oberiz hasta ellas.
Should Oberiz move and rename downloads?|¿Quieres que Oberiz mueva y renombre las descargas?
Yes|Sí
Oberiz imports each completed download into your library folder and renames it.|Oberiz importa cada descarga terminada a la carpeta de tu biblioteca y la renombra.
No|No
qBittorrent downloads straight into your library and your media server (Cinetta) tells Oberiz what you have.|qBittorrent descarga directamente en tu biblioteca y tu servidor multimedia (Cinetta) le dice a Oberiz lo que tienes.
Oberiz will never open, move, rename or delete files. Point each qBittorrent category directly at a library folder. Folders, NAS access and path mappings are not needed for downloads.|Oberiz nunca abrirá, moverá, renombrará ni borrará archivos. Apunta cada categoría de qBittorrent directamente a una carpeta de la biblioteca. Para las descargas no hacen falta carpetas, acceso al NAS ni correspondencias de rutas.
Where does qBittorrent run?|¿Dónde está qBittorrent?
On this computer|En este equipo
Oberiz and qBittorrent see the same paths. Nothing to configure.|Oberiz y qBittorrent ven las mismas rutas. No hay que configurar nada.
On another computer or the NAS|En otro equipo o en el NAS
qBittorrent reports its own paths, which Oberiz must translate into folders it can open.|qBittorrent indica sus propias rutas, y Oberiz tiene que traducirlas a carpetas que pueda abrir.
Example: qBittorrent sees /volume1/WD19TB and Oberiz sees|Ejemplo: qBittorrent ve /volume1/WD19TB y Oberiz ve
Detecting…|Detectando…
Detect automatically|Detectar automáticamente
Suggested mappings|Correspondencias sugeridas
Checked: {0} opens correctly.|Comprobada: {0} se abre correctamente.
Not verified: {0} could not be opened. If the NAS needs a password, connect it in the NAS access step first.|Sin comprobar: no se pudo abrir {0}. Si el NAS pide contraseña, conéctalo primero en el paso de acceso al NAS.
Add selected mappings|Añadir las correspondencias seleccionadas
Folders|Carpetas
Where qBittorrent leaves completed downloads, as Oberiz sees it.|Dónde deja qBittorrent las descargas terminadas, tal como lo ve Oberiz.
Where movies end up, already renamed.|Dónde quedan las películas, ya renombradas.
Where series end up, already renamed.|Dónde quedan las series, ya renombradas.
NAS access (SMB)|Acceso al NAS (SMB)
The NAS username and password, so the Oberiz service can open folders that start with \\. Not needed for folders on this computer.|El usuario y la contraseña del NAS, para que el servicio de Oberiz pueda abrir las carpetas que empiezan por \\. No hace falta para carpetas de este equipo.
Configure NAS access (SMB)|Configurar acceso al NAS (SMB)
Advanced options|Opciones avanzadas
Existing library (optional)|Biblioteca existente (opcional)
Folders Oberiz scans to detect what you already had. Not needed if your media server (Cinetta) reports your library.|Carpetas que Oberiz escanea para detectar lo que ya tenías. No hace falta si tu servidor multimedia (Cinetta) informa de tu biblioteca.
new path mapping|nueva correspondencia de ruta
Create the path mappings these folders need automatically|Crear automáticamente las correspondencias de rutas que necesitan estas carpetas
If qBittorrent runs on another computer or the NAS, set its path mappings in Files and storage.|Si qBittorrent está en otro equipo o en el NAS, configura sus correspondencias de rutas en Archivos y almacenamiento.
Oberiz does not manage files: completed downloads stay where qBittorrent saved them, and seed cleanup never deletes their data. Change it in Files and storage.|Oberiz no gestiona archivos: las descargas terminadas se quedan donde las guardó qBittorrent y la limpieza tras la siembra nunca borra sus datos. Se cambia en Archivos y almacenamiento.
Indexer folders|Carpetas de indexadores
Where tracker definitions are read from. Normally there is no need to change them.|De dónde se leen las definiciones de los trackers. Normalmente no hace falta cambiarlas.
Your private tracker definitions (.yml). They are never published or synchronized.|Tus definiciones privadas de trackers (.yml). Nunca se publican ni se sincronizan.
Public definitions Oberiz downloads when you synchronize the catalogue.|Definiciones públicas que Oberiz descarga al sincronizar el catálogo.
Forget this saved SMB connection? Existing Windows sessions will not be disconnected.|¿Olvidar esta conexión SMB guardada? No se desconectarán las sesiones existentes de Windows.
SMB connection saved|Conexión SMB guardada
Use HTTPS when entering credentials remotely. Saved passwords are protected by Windows; shared folders appear in the folder picker.|Usa HTTPS al introducir credenciales remotamente. Windows protege las contraseñas guardadas; las carpetas compartidas aparecen en el selector.
Share name (optional)|Nombre del recurso compartido (opcional)
Leave empty to keep the saved password|Deja vacío para conservar la contraseña guardada
Connecting…|Conectando…
Connect and save|Conectar y guardar
No shares discovered|No se han encontrado recursos compartidos
All storage locations|Todos los discos y recursos
Folder path|Ruta de la carpeta
Reset|Restablecer
Import YAML|Importar YAML
Choose a YAML definition up to 512 KiB.|Selecciona una definición YAML de hasta 512 KiB.
Replace the existing custom definition? Saved credentials will be preserved.|¿Sustituir la definición personalizada existente? Se conservarán las credenciales guardadas.
Episodes|Episodios
{0} missing · {1} future|{0} pendientes · {1} futuros
Episodes|Episodios
{0} missing · {1} future|{0} pendientes · {1} futuros
Add folders|Añadir carpetas
Select current folder|Seleccionar esta carpeta
{0} folders selected|{0} carpetas seleccionadas
Add selected folders|Añadir carpetas seleccionadas
Maximum 64 folders per media type.|Máximo 64 carpetas por tipo de contenido.
Complete both paths in each mapping, or remove the row.|Completa ambas rutas de cada correspondencia o elimina la fila.
Movie import destination|Destino de importación de películas
Series import destination|Destino de importación de series
Library folders|Carpetas de la biblioteca
Library folder|Carpeta de la biblioteca
Add every disk containing existing media. Scanning never moves files.|Añade todos los discos con contenido existente. El escaneo nunca mueve archivos.
Remote path mappings|Correspondencias de rutas remotas
Map qBittorrent paths to folders accessible by the Oberiz server. Categories and torrent states are not changed.|Relaciona las rutas de qBittorrent con carpetas accesibles desde el servidor de Oberiz. No se cambian las categorías ni el estado de los torrents.
Add mapping|Añadir correspondencia
Add folder|Añadir carpeta
Load category folders|Cargar carpetas de categorías
Add from qBittorrent category|Añadir desde una categoría de qBittorrent
Schedule library scans|Programar escaneos de la biblioteca
Unavailable disks preserve indexed files and hold automatic acquisition to avoid duplicate downloads.|Si un disco no está disponible, se conservan sus archivos indexados y se detienen las búsquedas automáticas para evitar descargas duplicadas.
Accessible|Accesible
Not scanned yet|Todavía sin escanear
Monitored Series|Series monitorizadas
Available episodes|Episodios disponibles
{0} aired missing · {1} future|{0} emitidos pendientes · {1} futuros
Size of indexed media files, not disk capacity or all torrents.|Tamaño de los archivos multimedia indexados, no la capacidad de los discos ni todos los torrents.
Configure folders and scan|Configurar carpetas y escanear
Season complete|Temporada completa
Up to date|Al día
Partially available|Disponible parcialmente
Follow upcoming episodes|Seguir próximos episodios
Backup created successfully.|Copia de seguridad creada.
Backup uploaded successfully.|Copia de seguridad subida.
Backup deleted.|Copia de seguridad eliminada.
Backup restored successfully.|Copia de seguridad restaurada.
Connecting to Oberiz…|Conectando con Oberiz…
The server is starting or reconnecting. This page will continue automatically.|El servidor está iniciándose o reconectando. La página continuará automáticamente.
Remove configured indexer {0}? The definition remains in Add Indexer. Seed policies and download history are preserved.|¿Eliminar el indexador configurado {0}? La definición seguirá en Añadir indexador. Se conservarán las políticas de compartición y el historial.
Search priority|Prioridad de búsqueda
Custom qBittorrent tag (optional)|Etiqueta de qBittorrent personalizada (opcional)
RSS feed URL (optional)|URL del feed RSS (opcional)
Search priority must be a whole number|La prioridad debe ser un número entero
Search priority must be between 0 and 10000|La prioridad debe estar entre 0 y 10000
Password protection is not enabled|La protección por contraseña no está activada
Password is too long|La contraseña es demasiado larga
Too many failed sign-in attempts. Try again later.|Demasiados intentos de acceso fallidos. Inténtalo más tarde.
Incorrect password|Contraseña incorrecta
Current password is too long|La contraseña actual es demasiado larga
Current password is incorrect|La contraseña actual es incorrecta
Password must be between 8 and 128 characters|La contraseña debe tener entre 8 y 128 caracteres
Cleaned|Limpiado
Imported|Importado
Importing|Importando
Missing|Pendiente
Duplicate|Duplicado
Reseeded|Reseed iniciado
Reconciled|Recuperado
Downloading Metadata|Descargando metadatos
Pending|Pendiente
Copying|Copiando
Hardlinking|Creando enlace duro
paused|pausado
running|en ejecución
Never monitored|Sin monitorizar
Events|Eventos
Add Oberiz twice under|Añade Oberiz dos veces en
. Endpoints: GET /api/v1/status, POST/GET /api/v1/requests, GET /api/v1/requests/:id.|. Rutas: GET /api/v1/status, POST/GET /api/v1/requests, GET /api/v1/requests/:id.
Movies (Radarr): http://your-oberiz-host:2032/radarr|Películas (Radarr): http://tu-servidor-oberiz:2032/radarr
Series (Sonarr): http://your-oberiz-host:2032/sonarr|Series (Sonarr): http://tu-servidor-oberiz:2032/sonarr
Z:\WD19TB or \\nas\WD19TB|Z:\WD19TB o \\nas\WD19TB
e.g. my-private-tracker|p. ej. mi-tracker-privado
· job #|· trabajo #
resolutions|Resoluciones
sources|Fuentes
codecs|Códecs
hdr|HDR
audio|Audio
configured|configurado
disabled|desactivado
healthy|en buen estado
Update|Actualizar
planned|programado
pending|pendiente
downloading|descargando
completed|completado
imported|importado
cleaned|limpiado
error|error
rejected|rechazado
queued|en cola
copy|copia
hardlink|enlace duro
move|mover
Browse|Explorar
Only use RSS for this indexer (skip it in the scheduled automatic search)|Usar solo RSS para este indexador (omitir búsquedas automáticas programadas)
Resolutions|Resoluciones
Sources|Fuentes
Codecs|Códecs
Hdr|HDR
Audio|Audio
All|Todos
Offline|Desconectado
Spanish preferred|Español preferido
English preferred|Inglés preferido
Languages|Idiomas
Load more|Cargar más
Min|Mínimo
Max|Máximo
Error|Error
total|total
 · cutoff {0}| · límite {0}
Add {0}|Añadir {0}
Completed · {0} missing · {1} searched · {2} grabbed · {3} skipped · {4} errors · {5} monitored|Completado · {0} pendientes · {1} buscados · {2} enviados · {3} omitidos · {4} errores · {5} monitorizados
Connected · qBittorrent {0} · {1} ms · {2}|Conectado · qBittorrent {0} · {1} ms · {2}
Delete local backup {0}? This cannot be undone.|¿Borrar la copia local {0}? No se puede deshacer.
Delete {0}?|¿Borrar {0}?
Downloading {0}%|Descargando {0}%
Movie · {0}|Película · {0}
RSS run · {0} feeds · {1} new · {2} matched · {3} grabbed · {4} errors · {5} configured|Ejecución RSS · {0} feeds · {1} nuevos · {2} coincidencias · {3} enviados · {4} errores · {5} configurados
Remove selected{0}|Eliminar seleccionados{0}
Remove the category “{0}”? Torrents will remain in qBittorrent but become uncategorized.|¿Eliminar la categoría «{0}»? Los torrents permanecerán en qBittorrent sin categoría.
Remove the tag “{0}” from qBittorrent?|¿Eliminar la etiqueta «{0}» de qBittorrent?
Remove {0}|Eliminar {0}
Remove {0} obsolete Oberiz record{1}? qBittorrent torrents and media files will not be changed.|¿Eliminar {0} registros obsoletos de Oberiz? No se modificarán los torrents de qBittorrent ni los archivos multimedia.
Removed {0}|Eliminado {0}
Reseed started in {0}|Reseed iniciado en {0}
Restore {0}? Library data, requests, preferences and history will be restored. Current security credentials and connections are preserved.|¿Restaurar {0}? Se restaurarán el contenido, peticiones, preferencias e historial. Se conservan la contraseña, claves y conexiones actuales.
Running · {0}/{1} · {2}|En ejecución · {0}/{1} · {2}
Search {0}...|Buscar {0}...
Season {0}|Temporada {0}
Series · {0}|Serie · {0}
Updated {0}|Actualizado {0}
every {0} min|cada {0} min
{0} / {1} enabled|{0} / {1} activados
{0} Copy|{0} Copia
{0} Reloading…|{0} Recargando…
{0} Season {1}|{0} Temporada {1}
{0} d ago|hace {0} días
{0} days ago|hace {0} días
{0} enabled (not a live health check)|{0} activados (no es una comprobación en tiempo real)
{0} episodes|{0} episodios
{0} feed{1} · {2}|{0} feeds · {2}
{0} h ago|hace {0} h
{0} historical issue{1}|{0} incidencias históricas
{0} hours ago|hace {0} horas
{0} min ago|hace {0} min
{0} minutes ago|hace {0} minutos
{0} missing episode{1}|{0} episodios pendientes
{0} record{1} removed.|{0} registros eliminados.
{0} season{1}|{0} temporadas
{0} · ETA {1} · {2} seeds|{0} · quedan {1} · {2} semillas
{0} · {1} queued|{0} · {1} en cola
{0}% average|{0}% de media
{0}% covered|{0}% cubierto
{0}% healthy|{0}% en buen estado
{0}/{1} available|{0}/{1} disponibles
{0}/{1} processed|{0}/{1} procesados
{0}: {1}/{2} matched|{0}: {1}/{2} asociados
{0}h {1}m left|quedan {0} h {1} min
{0}m left|quedan {0} min
Indexers|Indexadores
Downloads|Descargas
Movies|Películas
Series|Series
Profiles|Perfiles
Settings|Ajustes
Appearance|Apariencia
Theme|Tema
Dark|Oscuro
Middle|Intermedio
Light|Claro
Add Media|Añadir contenido
History|Historial
Calendar|Calendario
Imports & Reseed|Importación y reseed
Notifications & activity|Notificaciones y actividad
Search Oberiz...|Buscar en Oberiz...
Portal language|Idioma del portal
Stored permanently in Oberiz settings.|Se guarda en los ajustes de Oberiz.
Services, automation, importing, seeding and appearance.|Servicios, automatización, importación, compartición y apariencia.
Status unknown|Estado desconocido
Needs attention|Necesita atención
All Systems Operational|Todos los sistemas operativos
% of library|% de la biblioteca
(optional)|(opcional)
+ tags|+ etiquetas
, using the API key above:|, usando la clave API anterior:
. Library metadata stays usable; download progress is paused.|. Los metadatos de la biblioteca siguen disponibles; el progreso de descarga está pausado.
. Restoring replaces all current database data; media files and qBittorrent are not changed.|. La restauración reemplaza todos los datos de la base de datos; no modifica los archivos multimedia ni qBittorrent.
. Season packs are imported file-by-file when episode numbers can be detected.|. Los packs de temporada se importan archivo por archivo cuando se detectan los números de episodio.
. Series also supports|. Las series también admiten
14 days|14 días
30 days|30 días
45 days|45 días
90 days|90 días
ADD INDEXER|AÑADIR INDEXADOR
API key|Clave API
Accept complete series|Aceptar series completas
Action|Acción
Actions|Acciones
Active Downloads|Descargas activas
Active Indexers|Indexadores activos
Activity History|Historial de actividad
Add Indexer|Añadir indexador
Add Magnet|Añadir magnet
Add a movie or series to start tracking wanted media here.|Añade una película o serie para empezar a monitorizar el contenido pendiente.
Add magnet|Añadir magnet
Add media|Añadir contenido
Add to qBittorrent|Añadir a qBittorrent
Added|Añadido
Admin Password|Contraseña de administrador
Air Date|Fecha de emisión
All (|Todos (
All Availability|Cualquier disponibilidad
All Episodes|Todos los episodios
All Monitoring|Cualquier monitorización
All Quality|Cualquier calidad
All Series|Todas las series
Allow Oberiz to pick this release again|Permitir que Oberiz vuelva a elegir esta copia
Allow releases where language cannot be identified|Permitir copias cuyo idioma no se pueda identificar
Allow unknown resolution|Permitir resolución desconocida
Allow unknown source|Permitir fuente desconocida
Allow upgrades|Permitir mejoras
Allowed languages and language-specific scoring.|Idiomas permitidos y puntuaciones por idioma.
Also delete downloaded files|Borrar también los archivos descargados
At least 8 characters|Al menos 8 caracteres
Auto — hardlink, then copy fallback|Automático: enlace duro y, si falla, copia
Automatically search monitored media using its assigned profile.|Buscar automáticamente contenido monitorizado con su perfil asignado.
Automation|Automatización
Automation scope|Alcance de la automatización
Availability|Disponibilidad
Available|Disponible
Backup & Restore|Copias de seguridad y restauración
Backups are stored in|Las copias se guardan en
Bandwidth Usage|Uso del ancho de banda
Blank = downloads/reseed|Vacío = downloads/reseed
Both|Ambos
Build your library|Crea tu biblioteca
Cancel|Cancelar
Categories|Categorías
Category|Categoría
Category / Tags|Categoría / Etiquetas
Category controls destination. Tags can use variables.|La categoría determina el destino. Las etiquetas admiten variables.
Category name|Nombre de categoría
Cleanup|Limpieza
Cleanup downloads after seed policy is satisfied|Limpiar descargas al cumplir la política de compartición
Clients send|Los clientes envían
Clone|Clonar
Close|Cerrar
Close menu|Cerrar menú
Comma separated. Variables are resolved at Grab time.|Separadas por comas. Las variables se resuelven al descargar.
Common tokens:|Variables habituales:
Completed|Completado
Completed (|Completados (
Completed Today|Completados hoy
Completed downloads are imported without breaking active seeding.|Las descargas completadas se importan sin interrumpir la compartición.
Completed imports, seed cleanup and recoverable torrent metadata.|Importaciones completadas, limpieza de compartición y metadatos para reseed.
Configure Overseerr|Configurar Overseerr
Configured Indexers|Indexadores configurados
Configured indexers first. Definitions stay in the Add Indexer catalogue.|Primero los indexadores configurados. Las definiciones siguen en el catálogo de Añadir indexador.
Confirm new password|Confirmar nueva contraseña
Confirm password|Confirmar contraseña
Consistent SQLite snapshots, including data currently in WAL.|Copias coherentes de SQLite, incluidos los datos del registro WAL.
Controls which episodes automation should look for.|Determina qué episodios debe buscar la automatización.
Copy|Copiar
Copy this key now.|Copia esta clave ahora.
Create|Crear
Create Backup|Crear copia de seguridad
Create an admin password|Crear contraseña de administrador
Create global qBittorrent tags for manual organization.|Crear etiquetas globales de qBittorrent para organizar manualmente.
Credential|Credencial
Current jobs|Trabajos actuales
Current password|Contraseña actual
Custom definitions|Definiciones personalizadas
Custom indexers|Indexadores personalizados
DOWNLOAD CLIENT|CLIENTE DE DESCARGA
DOWNLOAD ROUTING|DESTINO DE DESCARGA
Dark theme|Tema oscuro
Default|Predeterminado
Default Series Profile|Perfil predeterminado de series
Default profile|Perfil predeterminado
Default qBittorrent Category|Categoría predeterminada de qBittorrent
Default save path|Ruta de guardado predeterminada
Definition Sources|Fuentes de definiciones
Definition catalogue|Catálogo de definiciones
Delete|Borrar
Delete Series|Borrar serie
Details/profile|Detalles/perfil
Diagnostics|Diagnóstico
Direct integration for your multimedia app or other request clients.|Integración directa para tu aplicación multimedia u otros clientes de peticiones.
Discover existing media files and rebuild real availability.|Detectar archivos existentes y actualizar su disponibilidad real.
Dismiss|Descartar
Download|Descargar
Download client|Cliente de descarga
Download filters|Filtros de descargas
Download speed|Velocidad de descarga
Downloaded|Descargado
Downloading|Descargando
Downloading (|Descargando (
Downloads (|Descargas (
ETA|Tiempo restante
Each category can have its own default save path.|Cada categoría puede tener su propia ruta de guardado predeterminada.
Each movie or series can use a different profile.|Cada película o serie puede usar un perfil diferente.
Edit|Editar
Either|Cualquiera
Enable Overseerr compatibility|Activar compatibilidad con Overseerr
Enable RSS feed sync|Activar sincronización RSS
Enable automatic searches|Activar búsquedas automáticas
Enable public request API|Activar API pública de peticiones
Enabled|Activado
English|Inglés
Episode|Episodio
Episodes Monitored|Episodios monitorizados
Episodes in range|Episodios en el intervalo
Example: category|Ejemplo: categoría
Existing Episodes|Episodios existentes
First Season|Primera temporada
Future Episodes|Episodios futuros
Future monitored episodes|Episodios futuros monitorizados
Future monitored episodes will appear here.|Aquí aparecerán los episodios futuros monitorizados.
Go to dashboard|Ir al inicio
Hardlink only|Solo enlace duro
Host|Servidor
INDEXER SETTINGS|AJUSTES DEL INDEXADOR
If Oberiz is useful to you, you can support its development through PayPal.|Si Oberiz te resulta útil, puedes apoyar su desarrollo mediante PayPal.
Import & Naming|Importación y nombres
Import completed downloads|Importar descargas completadas
Import method|Método de importación
Imports, searches and downloads will be recorded here.|Aquí se registrarán las importaciones, búsquedas y descargas.
Independent movie, series and language rules for matching, scoring and automatic downloads.|Reglas independientes de películas, series e idiomas para coincidencias, puntuaciones y descargas automáticas.
Indexed Storage|Almacenamiento indexado
Indexer|Indexador
Indexer Activity|Actividad de indexadores
Indexer ID|ID del indexador
Indexers (|Indexadores (
Inherit:|Heredar:
Interval (hours)|Intervalo (horas)
Interval (minutes)|Intervalo (minutos)
Keep .torrent + mapping for future reseed|Conservar .torrent y asociación para reseed
Keep latest copies|Copias recientes que conservar
LANGUAGE|IDIOMA
LANGUAGE PROFILE|PERFIL DE IDIOMA
LIBRARY|BIBLIOTECA
Language|Idioma
Language Profiles|Perfiles de idioma
Language profile|Perfil de idioma
Last 90 seconds|Últimos 90 segundos
Last Check|Última comprobación
Latency|Latencia
Latest Season|Última temporada
Library|Biblioteca
Library Maintenance|Mantenimiento de biblioteca
Library Overview|Resumen de biblioteca
Library entries|Elementos de biblioteca
Light theme|Tema claro
Live torrent activity from qBittorrent.|Actividad de torrents en tiempo real de qBittorrent.
Loaded|Cargado
Loading qBittorrent…|Cargando qBittorrent…
Loading seasons…|Cargando temporadas…
Loading settings…|Cargando ajustes…
Local path (as Oberiz sees it)|Ruta local (visible para Oberiz)
Lower priority runs first and breaks ties between equally scored releases. The custom tag replaces|La prioridad más baja se ejecuta primero y desempata copias con igual puntuación. La etiqueta personalizada sustituye
MANUAL SEARCH|BÚSQUEDA MANUAL
Magnet URL|Enlace magnet
Manage categories, save paths and tags|Gestionar categorías, rutas y etiquetas
Manage your movie collection, downloads and monitoring|Gestiona tus películas, descargas y monitorización
Manual|Manual
Max episode size MB|Tamaño máximo de episodio (MB)
Max season/pack size MB|Tamaño máximo de temporada/pack (MB)
Media|Contenido
Media type|Tipo de contenido
Media, downloads, reseed workspace and metadata.|Contenido, descargas, espacio de reseed y metadatos.
Metadata, posters and media information|Metadatos, pósteres e información del contenido
Method|Método
Middle theme|Tema intermedio
Min seeds|Semillas mínimas
Min size MB|Tamaño mínimo (MB)
Minimum ratio|Ratio mínimo
Minimum seed days|Días mínimos de compartición
Minimum seeds|Semillas mínimas
Missing|Pendiente
Missing Episodes|Episodios pendientes
Missing · quality upgrades|Pendientes · mejoras de calidad
Monitor|Monitorizar
Monitor Mode|Modo de monitorización
Monitored|Monitorizado
Monitored Movies|Películas monitorizadas
Monitored only|Solo monitorizados
More|Más
More series options|Más opciones de series
Move (only for setups without seeding)|Mover (solo si no se comparte)
Movie|Película
Movie Profiles|Perfiles de películas
Movie naming template|Plantilla de nombre de películas
Movies Library|Biblioteca de películas
Movies and series metadata|Metadatos de películas y series
NEW DOWNLOAD|NUEVA DESCARGA
Name|Nombre
Needs Test|Pendiente de prueba
Never auto-delete|No borrar automáticamente
Never pick this release again for this title|No volver a elegir esta copia para este título
New Language Profile|Nuevo perfil de idioma
New Profile|Nuevo perfil
New grabs will appear here with their live progress.|Las nuevas descargas aparecerán aquí con su progreso en tiempo real.
New password|Nueva contraseña
New tag|Nueva etiqueta
No Oberiz import jobs yet.|Todavía no hay trabajos de importación de Oberiz.
No RSS synchronization has run yet.|Todavía no se ha ejecutado una sincronización RSS.
No backups created yet.|Todavía no hay copias de seguridad.
No categories configured in qBittorrent.|No hay categorías configuradas en qBittorrent.
No completed torrents yet.|Todavía no hay torrents completados.
No episodes in this calendar window.|No hay episodios en este intervalo del calendario.
No events yet.|Todavía no hay eventos.
No indexers configured yet. Use|Todavía no hay indexadores configurados. Usa
No movies match the current filters.|Ninguna película coincide con los filtros actuales.
No movies or series in your library match “|Ninguna película o serie de tu biblioteca coincide con «
No profile|Sin perfil
No recent activity|Sin actividad reciente
No recent errors recorded.|No hay errores recientes registrados.
No series match the current filters.|Ninguna serie coincide con los filtros actuales.
No subfolders.|No hay subcarpetas.
No tags configured in qBittorrent.|No hay etiquetas configuradas en qBittorrent.
No upcoming episodes|Sin próximos episodios
None|Ninguno
Not Monitored|Sin monitorizar
Oberiz is free to use. Donations are entirely optional.|Oberiz es gratuito. Las donaciones son totalmente opcionales.
Oberiz stores only its hash, not the key itself — once you leave or reload this page it can't be shown again. Generate a new one if you lose it.|Oberiz solo guarda el hash de la clave. Al salir o recargar esta página no podrá mostrarse de nuevo. Genera otra si la pierdes.
Oberiz will not pick these again for the title shown. Undo one to let it be chosen again.|Oberiz no volverá a elegir estas copias para el título indicado. Deshaz el rechazo para permitirlo.
On Hold|En espera
Online|En línea
Only configured indexers appear here.|Aquí solo aparecen los indexadores configurados.
Only needed when qBittorrent runs on a different machine than Oberiz. Leave both blank if they share the same filesystem.|Solo necesario si qBittorrent está en otro equipo. Deja ambas rutas vacías si comparten el sistema de archivos.
Open calendar|Abrir calendario
Open history|Abrir historial
Open imports|Abrir importaciones
Open menu|Abrir menú
Open notifications|Abrir notificaciones
Open seasons|Abrir temporadas
Open settings|Abrir ajustes
Open system diagnostics|Abrir diagnóstico del sistema
Optional series identifier lookup for compatible indexers|Consulta opcional de identificadores de series para indexadores compatibles
Password|Contraseña
Paths|Rutas
PayPal donation QR code for Oberiz|Código QR para donar a Oberiz mediante PayPal
Peers|Pares
Per-indexer cleanup rules. No policy means no automatic deletion.|Reglas de limpieza por indexador. Sin política no se borra automáticamente.
Port|Puerto
Prefer season/complete packs|Preferir packs de temporada o series completas
Preferred terms (TERM:score)|Términos preferidos (TÉRMINO:puntuación)
Prioritize indexer|Priorizar indexador
Priority|Prioridad
Process only new feed releases from configured indexers; each release is deduplicated before matching.|Procesar solo las nuevas copias de los feeds configurados; se eliminan duplicados antes de buscar coincidencias.
Profile|Perfil
Profile:|Perfil:
Progress|Progreso
Protects the whole Oberiz interface and API when it is reachable on your network.|Protege la interfaz y API de Oberiz cuando son accesibles desde tu red.
Prowlarr upstream|Catálogo de Prowlarr
Public API v1|API pública v1
QUALITY PROFILE|PERFIL DE CALIDAD
Quality Profile|Perfil de calidad
Quality Profiles|Perfiles de calidad
Queue Health|Estado de la cola
Queued|En cola
RSS Automation|Automatización RSS
RSS feed URL|URL del feed RSS
Read-only status for support and troubleshooting. Secrets are never included.|Estado de solo lectura para diagnóstico y soporte. No incluye credenciales.
Real events recorded by Oberiz.|Eventos reales registrados por Oberiz.
Recent Activity|Actividad reciente
Recent Completed|Completados recientes
Recent Downloads|Descargas recientes
Recent errors:|Errores recientes:
Recently Added|Añadidos recientemente
Recently Added / Wanted|Añadidos recientemente / Pendientes
Refresh|Actualizar
Refresh diagnostics|Actualizar diagnóstico
Reject|Rechazar
Reject release|Rechazar copia
Reject terms|Términos rechazados
Reject this release: remove it and never pick it again|Rechazar esta copia: eliminarla y no volver a elegirla
Rejected releases (|Copias rechazadas (
Reload|Recargar
Remote path (as qBittorrent sees it)|Ruta remota (visible para qBittorrent)
Remove|Eliminar
Remove from configured indexers|Eliminar de los indexadores configurados
Remove password|Eliminar contraseña
Remove torrent + original download|Eliminar torrent y descarga original
Remove torrent, keep original download|Eliminar torrent y conservar descarga original
Rename imported files|Renombrar archivos importados
Required to use Oberiz. It keeps you signed in on this browser for 30 days; changing or removing it needs the current password.|Necesaria para usar Oberiz. Mantiene la sesión durante 30 días en este navegador; cambiarla o eliminarla requiere la contraseña actual.
Requirement|Requisito
Rescan All|Reescanear todo
Rescan Movies|Reescanear películas
Rescan Series|Reescanear series
Rescan does not move or rename files. It indexes files already present in the configured library paths, refreshes Movies availability and links SxxExx files to Series episodes.|El escaneo no mueve ni renombra archivos. Indexa los existentes, actualiza la disponibilidad de películas y asocia archivos SxxExx con episodios.
Reseed workspace|Espacio de reseed
Restore|Restaurar
Runtime tests and failures will appear here.|Aquí aparecerán las pruebas y errores de ejecución.
SERIES|SERIES
Save|Guardar
Save Language Profile|Guardar perfil de idioma
Save Policy|Guardar política
Save Profile|Guardar perfil
Save Settings|Guardar ajustes
Save path (optional)|Ruta de guardado (opcional)
Schedule automatic backups|Programar copias de seguridad automáticas
Score|Puntuación
Search|Buscar
Search Pack|Buscar pack
Search Prowlarr and custom definitions...|Buscar definiciones de Prowlarr y personalizadas...
Search configured indexers...|Buscar indexadores configurados...
Search downloads|Buscar descargas
Search downloads...|Buscar descargas...
Search movies...|Buscar películas...
Search releases|Buscar copias
Search series...|Buscar series...
Searching enabled indexers…|Buscando en los indexadores activados…
Seed Policies|Políticas de compartición
Seeding|Compartiendo
Seeding (|Compartiendo (
Seeds|Semillas
Seeds / Peers|Semillas / Pares
Select|Seleccionar
Select folder|Seleccionar carpeta
Self-hosted Media Automation|Automatización multimedia autoalojada
Series Library|Biblioteca de series
Series Profiles|Perfiles de series
Series identifiers for episode matching|Identificadores de series para asociar episodios
Series naming template|Plantilla de nombre de series
Series release search|Búsqueda de copias de series
Set Default|Predeterminar
Set a password once to protect this Oberiz instance before using it.|Configura una contraseña para proteger esta instancia antes de usarla.
Set an optional|Configura una
Settings → Services|Ajustes → Servicios
Show rejected releases|Mostrar copias rechazadas
Sign in|Iniciar sesión
Sign out|Cerrar sesión
Size|Tamaño
Sort|Ordenar
Sort by: Date Added|Ordenar por: fecha de incorporación
Sort by: Recently Added|Ordenar por: añadidos recientemente
Sort by: Title|Ordenar por: título
Sort by: Year|Ordenar por: año
Sort downloads by|Ordenar descargas por
Speed|Velocidad
Stalled|Sin actividad
Status|Estado
Support Oberiz|Apoya a Oberiz
Support via PayPal|Apoyar mediante PayPal
Sync Now|Sincronizar ahora
System Status|Estado del sistema
TMDB is configured independently by each user with their own API credential and subject to TMDB’s terms.|Cada usuario configura TMDB con sus propias credenciales API y conforme a sus condiciones.
Tags|Etiquetas
Test|Probar
Test Connection|Probar conexión
The profile controls quality, language, upgrades and qBittorrent routing.|El perfil controla la calidad, idioma, mejoras y destino en qBittorrent.
The torrent is removed from qBittorrent and Oberiz will never pick this release again for this title; its next search will look for another one. You can undo it from the manual search.|Se elimina el torrent de qBittorrent y Oberiz no volverá a elegir esta copia para este título. La siguiente búsqueda elegirá otra. Puedes deshacerlo desde la búsqueda manual.
This Oberiz instance is protected with an admin password.|Esta instancia está protegida con una contraseña de administrador.
This permanently removes the data from disk.|Esto borra permanentemente los datos del disco.
This record has no reusable .torrent metadata and library path.|Este registro no tiene metadatos .torrent reutilizables ni ruta de biblioteca.
Time|Hora
Title|Título
Torrent metadata|Metadatos del torrent
Total Movies|Total de películas
Total Series|Total de series
Total Speed|Velocidad total
Track seasons and episodes with inherited profiles and monitoring rules|Gestiona temporadas y episodios con perfiles y reglas de monitorización heredados
Tracker / source|Tracker / fuente
Type|Tipo
Unavailable|No disponible
Uncategorized|Sin categoría
Uncategorized / Sin categorizar|Sin categoría
Undo|Deshacer
Undo reject|Deshacer rechazo
Unknown language:|Idioma desconocido:
Upcoming|Próximos
Upcoming Episodes|Próximos episodios
Upcoming and recently aired episodes from your Oberiz library.|Episodios próximos y recién emitidos de tu biblioteca.
Upcoming missing|Próximos pendientes
Upgrade cutoff score|Puntuación límite de mejoras
Upgrade wanted|Mejora pendiente
Upgrades wanted|Mejoras pendientes
Upload backup|Subir copia de seguridad
Upload speed|Velocidad de subida
Upstream indexers|Indexadores del catálogo
Use HTTPS|Usar HTTPS
Use the same key in both services. The movie and series root folders and quality profiles come from Oberiz.|Usa la misma clave en ambos servicios. Las carpetas raíz y perfiles de calidad proceden de Oberiz.
Use this folder|Usar esta carpeta
Use this indexer in release searches.|Usar este indexador en las búsquedas.
Used only to resolve a TVDB series ID for indexers that request it. It is never sent to indexers.|Solo se usa para obtener el ID de TVDB de la serie cuando un indexador lo solicita. La clave no se envía a los indexadores.
Username|Usuario
View All|Ver todo
View Calendar|Ver calendario
Wanted|Pendiente
Wanted / Upgrades|Pendientes / Mejoras
Your configured indexers|Tus indexadores configurados
Your download queue is clear|Tu cola de descargas está vacía
accepted ·|aceptados ·
added|añadido
available ·|disponibles ·
can be removed safely. Use Shift to select a range.|se pueden eliminar. Usa Mayús para seleccionar un intervalo.
configured feeds · last interval:|feeds configurados · último intervalo:
days · ratio|días · ratio
definitions available to configure|definiciones disponibles para configurar
enabled|activados
enabled not tested yet|activados sin probar
enabled online|activados en línea
enabled · Automation:|activados · Automatización:
episodes|episodios
files|archivos
grabbed|enviados
in each Indexer configuration. Existing releases are remembered, so a feed item is evaluated once only.|en cada indexador. Se recuerdan las copias existentes para evaluar cada elemento una sola vez.
in the selected quality profile’s tag template. "Only use RSS" still lets this indexer be searched when you add a title or click Search — it's skipped only by the scheduled automatic search.|en la plantilla de etiquetas del perfil. «Solo RSS» permite buscar al añadir un título o pulsar Buscar; solo omite las búsquedas automáticas programadas.
indexer errors|errores de indexadores
language profiles|perfiles de idioma
matched ·|coincidencias ·
monitored|monitorizados
movies|películas
new ·|nuevos ·
obsolete record|registro obsoleto
of|de
profiles|perfiles
qBittorrent Organization|Organización de qBittorrent
qBittorrent Tags|Etiquetas de qBittorrent
qBittorrent connection error|Error de conexión con qBittorrent
qBittorrent unavailable:|qBittorrent no disponible:
rejected ·|rechazados ·
season|temporada
seeds|semillas
shown|mostrados
to choose one from the definition catalogue.|para elegir uno del catálogo de definiciones.
total in library|total en biblioteca
· Database:|· Base de datos:
· Indexers:|· Indexadores:
← Back|← Volver
↑ Parent folder|↑ Carpeta superior
→ qBittorrent gets the destination category and tags such as|→ qBittorrent recibe la categoría de destino y etiquetas como
↻ Refresh|↻ Actualizar
API key copied to clipboard.|Clave API copiada al portapapeles.
API key or Read Access Token|Clave API o token de acceso de lectura
API key saved. Copy it now — it will not be shown again.|Clave API guardada. Cópiala ahora; no volverá a mostrarse.
Action failed|La acción ha fallado
Active records cannot be removed|No se pueden eliminar registros activos
Adding…|Añadiendo…
Admin password removed.|Contraseña de administrador eliminada.
Admin password saved. Use it next time Oberiz asks you to sign in.|Contraseña guardada. Úsala cuando Oberiz solicite iniciar sesión.
All episodes|Todos los episodios
Ascending|Ascendente
Ascending sort|Orden ascendente
Audio|Audio
Automation checked media|Contenido comprobado por la automatización
Automation needs attention|La automatización necesita atención
Browse folders|Explorar carpetas
Change password|Cambiar contraseña
Check Settings connection|Comprobar conexión en Ajustes
Check download queue|Comprobar cola de descargas
Checking|Comprobando
Choose another default profile before deleting|Elige otro perfil predeterminado antes de borrar
Choose how Oberiz should manage this title.|Elige cómo gestionar este título.
Clone profile|Clonar perfil
Collapse menu|Contraer menú
Collecting diagnostic data…|Recopilando datos de diagnóstico…
Configured|Configurado
Connected|Conectado
Continue|Continuar
Copied|Copiado
Could not access clipboard. Select and copy the key manually.|No se pudo acceder al portapapeles. Selecciona y copia la clave manualmente.
Could not add magnet|No se pudo añadir el magnet
Could not create category|No se pudo crear la categoría
Could not create tag|No se pudo crear la etiqueta
Could not delete torrent|No se pudo borrar el torrent
Could not load downloads|No se pudieron cargar las descargas
Could not load indexers|No se pudieron cargar los indexadores
Could not load qBittorrent organization|No se pudo cargar la organización de qBittorrent
Could not reach qBittorrent|No se pudo conectar con qBittorrent
Could not reject release|No se pudo rechazar la copia
Could not remove category|No se pudo eliminar la categoría
Could not remove tag|No se pudo eliminar la etiqueta
Could not undo the rejection|No se pudo deshacer el rechazo
Could not update category|No se pudo actualizar la categoría
Could not update torrent organization|No se pudo actualizar la organización del torrent
Creating backup…|Creando copia de seguridad…
Current|Actual
DELETE|BORRAR
DELETE TORRENT|BORRAR TORRENT
Delete torrent|Borrar torrent
Delete torrent + files|Borrar torrent y archivos
Deleting backup…|Borrando copia de seguridad…
Descending|Descendente
Descending sort|Orden descendente
Disable|Desactivar
Disabled|Desactivado
Edit language profile|Editar perfil de idioma
Edit profile|Editar perfil
Enable|Activar
Everything looks good|Todo funciona correctamente
Existing episodes|Episodios existentes
Expand menu|Expandir menú
Expand to load episode metadata|Expandir para cargar los metadatos de episodios
Finished|Finalizado
First season|Primera temporada
For example: ThePirateBay|Por ejemplo: ThePirateBay
Future episodes|Episodios futuros
Games|Juegos
Generate key|Generar clave
Generate or enter a key|Genera o introduce una clave
Grab|Descargar
Hide|Ocultar
Idle|Inactivo
Import needs attention|La importación necesita atención
Imported to library|Importado a biblioteca
Indexer ID is required|El ID del indexador es obligatorio
Keep current if blank|Vacío para conservar el actual
Keep this series under automation.|Mantener esta serie en la automatización.
Latest season|Última temporada
Leave blank to keep current|Vacío para conservar el actual
Loaded directly from qBittorrent. Empty means truly uncategorized; save paths remain managed by qBittorrent.|Cargado desde qBittorrent. Vacío significa sin categoría; qBittorrent gestiona las rutas de guardado.
Loading…|Cargando…
Lower number is preferred|Un número menor tiene preferencia
MOVIE|PELÍCULA
Metadata|Metadatos
Missing episodes|Episodios pendientes
Moving|Moviendo
Never|Nunca
New Movie Profile|Nuevo perfil de películas
New Series Profile|Nuevo perfil de series
New key generated. Save Settings to activate it.|Clave generada. Guarda los ajustes para activarla.
New key generated. Save Settings to replace the current key.|Clave generada. Guarda los ajustes para sustituir la actual.
New language profile|Nuevo perfil de idioma
New profile|Nuevo perfil
Next episode|Próximo episodio
No active transfers|Sin transferencias activas
No air date|Sin fecha de emisión
No description available.|Sin descripción disponible.
No feeds configured|Sin feeds configurados
No matching downloads|No hay descargas coincidentes
No overview|Sin sinopsis
No overview available.|Sin sinopsis disponible.
No releases found in the enabled indexers.|No se encontraron copias en los indexadores activados.
No releases pass the selected profile.|Ninguna copia cumple el perfil seleccionado.
No tags|Sin etiquetas
No torrents yet|Todavía no hay torrents
Not configured|Sin configurar
Not tested|Sin probar
OFF|DESACTIVADO
ON|ACTIVADO
On hold|En espera
Password must be at least 8 characters|La contraseña debe tener al menos 8 caracteres
Passwords do not match|Las contraseñas no coinciden
Preparing next search…|Preparando la siguiente búsqueda…
Private|Privado
Public|Público
Queue needs attention|La cola necesita atención
REJECT RELEASE|RECHAZAR COPIA
RSS synchronization completed|Sincronización RSS completada
RSS synchronization failed|La sincronización RSS ha fallado
RUNNING|EN EJECUCIÓN
Refresh TMDB|Actualizar TMDB
Refreshing…|Actualizando…
Regenerate key|Regenerar clave
Reject + delete files|Rechazar y borrar archivos
Reject this release?|¿Rechazar esta copia?
Rejected|Rechazado
Reload Definitions|Recargar definiciones
Remove download?|¿Eliminar descarga?
Remove the admin password? Oberiz will ask you to create a new one the next time it's opened; there is no way to leave it unprotected.|¿Eliminar la contraseña de administrador? Oberiz pedirá crear otra al abrirlo; no puede quedar sin protección.
Removing…|Eliminando…
Restoring backup…|Restaurando copia de seguridad…
Run Now|Ejecutar ahora
Running automation and checking qBittorrent…|Ejecutando automatización y comprobando qBittorrent…
Running…|Ejecutando…
Saved|Guardado
Saving…|Guardando…
Scanning library…|Escaneando biblioteca…
Search TMDB for a movie or series.|Busca una película o serie en TMDB.
Search missing releases and allowed upgrades.|Buscar copias pendientes y mejoras permitidas.
Searching…|Buscando…
Select for removal — hold Shift to select a range|Seleccionar para eliminar; mantén Mayús para seleccionar un intervalo
Sending…|Enviando…
Sent to qBittorrent|Enviado a qBittorrent
Set password|Configurar contraseña
Set — generate a new key to replace it|Configurada; genera otra clave para sustituirla
Show|Mostrar
Signing in…|Iniciando sesión…
Spanish|Español
Specials|Especiales
Start|Iniciar
Starting…|Iniciando…
Stop|Detener
Stopped|Detenido
Syncing…|Sincronizando…
Testing…|Probando…
Today|Hoy
Torrent removed from qBittorrent|Torrent eliminado de qBittorrent
Try another filter or search term.|Prueba otro filtro o término de búsqueda.
Unknown|Desconocido
Unmonitor|Dejar de monitorizar
Unnamed torrent|Torrent sin nombre
Up to date|Al día
Updated|Actualizado
Uploading backup…|Subiendo copia de seguridad…
Waiting…|Esperando…
just now|ahora mismo
needs review|necesita revisión
not configured|sin configurar
qBittorrent is connected. Add a magnet or connect your main qBittorrent later.|qBittorrent está conectado. Añade un magnet o conecta tu qBittorrent principal más adelante.
status unavailable|estado no disponible
○ Disabled|○ Desactivado
○ Not configured|○ Sin configurar
○ Not monitored|○ Sin monitorizar
● Configured|● Configurado
● Enabled|● Activado
● Monitored|● Monitorizado
● Not tested|● Sin probar
● Offline|● Desconectado
● Online|● En línea
⚠ Missing|⚠ Pendiente
✓ Available|✓ Disponible
 · Default| · Predeterminado
 · Rejected| · Rechazado
 · currently configured| · configurado actualmente
! Missing|! Pendiente
`.trim().split('\n').map(line => {
  const separator = line.indexOf('|')
  return [line.slice(0, separator), line.slice(separator + 1)]
}))
