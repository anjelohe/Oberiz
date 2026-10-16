# Oberiz — Memoria del proyecto

## Cierre posterior autorizado — v1.0.4, 2026-10-16

El usuario autoriza corregir los cuatro hallazgos y publicar. Corregidos: fijación
DNS de aliases autenticados y validación manual de login; consumo incremental de
metadatos; conservación del último informe válido del alcance durante evicción;
rangos de episodios sin desbordamiento y expansión limitada. Las notas de bloqueo
inferiores son históricas. Primera contraseña por red sin cambios. No se despliega
el servicio ni se reinicia qBittorrent; falta observar su memoria en uso real.

Verificación final: 124 pruebas unitarias correctas (una ignorada), dos de migración,
cinco suites frontend y compilación, formato y Clippy correctos. OSV: 252 versiones
activas sin avisos; npm: cero vulnerabilidades. Publicación autorizada en main con
etiqueta v1.0.4; los instaladores los genera GitHub Actions, no el equipo local.

## Revisión posterior a seguridad — 2026-10-16

Nuevos cambios recibidos y revisados sin modificar su código. Publicación 1.0.4
retenida: regresión de fijación DNS/validación de aliases de login, precarga de
metadatos sin cota acumulada, evicción del último informe válido del alcance
actual y desbordamiento de rangos de episodios. Ver hallazgos al inicio de la
auditoría. 120 pruebas unitarias, frontend, formato y Clippy pasan, pero no cubren
estos casos. No se ha creado commit/etiqueta ni enviado cambios a GitHub.

## Seguridad y qBittorrent — publicación autorizada 2026-10-16

Tras detener la publicación y completar la auditoría, el usuario autoriza subir
la versión 1.0.4 a GitHub. Correcciones solicitadas: sesiones HTTP
de qBittorrent reutilizadas, renovación serializada y consultas por hash; sondeos
de interfaz sin solapamiento. No cambia torrents ni preferencias de qBittorrent.
Filtro de IP y redirecciones reforzado; cuotas de memoria/archivo/informes e
intentos de login; herramientas Windows con rutas absolutas; saneado de errores.
Restaurar conserva identidad de seguridad y conexiones actuales, revocando las
sesiones como antes. Confirmación de interfaz actualizada en seis idiomas.
La PRIMERA CONTRASEÑA sigue configurable POR RED, explícitamente solicitado.
Subir código y etiqueta 1.0.4 para recompilar en GitHub. No actualizar el servicio
instalado ni reiniciar qBittorrent; esas acciones no forman parte de la publicación.
Ver [auditoría y cierre local](Oberiz_Auditoria_Seguridad_2026-10-16.md).
El consumo real de qBittorrent queda pendiente de medir; las sesiones antiguas
pueden persistir hasta caducar. No reiniciar qBittorrent automáticamente.

## Versión 1.0.4 — 2026-10-16

El usuario aprueba publicar los informes de faltantes y los ajustes visuales
probados en local. Versiones sincronizadas en frontend, backend, OpenAPI, Compose
y workflow. Ver [notas de versión](releases/v1.0.4.md).
Diseño aprobado: tarjetas sin borde en todos los temas, navegación SVG,
intermedio por defecto solo en nuevas instalaciones, favicon propio, cabecera
alineada y contenido 20 px más alto que el diseño previo. Panel con tres descargas
hasta 1920 px y cuatro por encima; columnas alineadas, botones homogéneos y suma
circular blanca en Añadir contenido. No reiniciar el servicio local ni modificar
datos privados para publicar. Adjuntos, copias y credenciales quedan fuera de Git.

## Faltantes — implementación incluida en 1.0.4, 2026-10-16

Motor, trabajos y endpoints implementados, separados de biblioteca/peticiones/
descargas. Inventario Cinetta library:read y metadatos TMDB, fechas <= hoy UTC.
Accesos en Peliculas, Series y cada titulo; conexion y permiso externo opt-in en
Ajustes. Ver [contrato y prueba local](Oberiz_Faltantes_Contrato.md).
Incluido en la publicación 1.0.4; la instalación local no se despliega automáticamente.

## Ajustes acordados — 2026-10-15 · v1.0.3

Primera contraseña configurable remotamente, sin exigir loopback; configurar en
red de confianza. Borrado manual de torrent con casilla de archivos desmarcada y
aviso permanente; automatismos conservan datos. Pausa por inventario Cinetta
ausente/antiguo opcional y desactivada por defecto, también al actualizar.
Barra lateral usa la versión de package.json en lugar de un literal obsoleto.
Estos ajustes sustituyen las restricciones de v1.0.2 descritas debajo.

## Cambio de arquitectura — 2026-10-15 · v1.0.2

La arquitectura de v1.0.2 está documentada en
[Almacenamiento simplificado](Oberiz_Almacenamiento_Simplificado.md).
Se retiran importación directa, copias/hardlinks/movimientos y conexiones SMB.
Inventario seleccionable: Cinetta por API o escaneo local de solo lectura, sin
cambio automático por desconexión. Fecha del último informe y retención de nuevas
adquisiciones si falta un inventario Cinetta de menos de 24 horas.
Renombrado opcional exclusivamente mediante qBittorrent, desactivado por defecto
y con consentimiento nuevo independiente del antiguo ajuste de importación.
Archivo `.torrent`/hash/destino/diario en SQLite, recuperación detenida y recheck,
sin arranque automático. Historial antiguo preservado; recuperación antigua manual.
No se cambian NAS, torrents reales, la instalación activa ni el repositorio Cinetta.
El emisor sí existe en `cinetta/src/services/oberizInventory*.js`; la revisión previa
limitada a Rust fue incompleta. Sus 11 pruebas sintéticas han pasado.
Prueba con qBittorrent 5.2.3 real en perfil aislado: renombrado, exportación/hash,
recuperación detenida y recheck correctos. Se detectó que su API puede sobrescribir
archivos ajenos; se añadió y probó una comprobación remota del directorio mediante
la API de qBittorrent, que bloquea destinos ocupados y falla de forma segura cuando
la consulta no está disponible. No hay garantía atómica frente a otros escritores.
Los apartados siguientes describen el estado publicado anterior, no el flujo nuevo.

Versión de producto: **1.0.3**. La descripción histórica de abajo parte de la primera versión **1.0.0**; los cambios del 15 de octubre están resumidos arriba. La publicación no actualiza automáticamente las instalaciones existentes.

## Propósito y arquitectura

Servidor autoalojado de automatización de películas y series: metadatos TMDB/TVDB, indexadores Cardigann, perfiles de calidad e idioma, búsqueda, RSS, qBittorrent, importación, biblioteca real, calendario, historial y API de peticiones.

Backend Rust/Axum/SQLite y frontend React/TypeScript. Distribución Windows (portable e instalador con servicio), Linux y Docker. Las migraciones SQLite se conservan para proteger datos existentes; reiniciar el historial del repositorio no implica reiniciar bases de datos o bibliotecas del usuario.

## Funciones incluidas

- Películas, series, temporadas y episodios con monitorización y perfiles heredados.
- Biblioteca en varios discos: carpetas independientes, selección múltiple, categorías de qBittorrent y correspondencias de rutas.
- Escaneo previo, manual y periódico configurable. No modifica archivos existentes; conserva disponibilidad cuando un disco no está accesible y evita descargas duplicadas.
- Estados de temporadas incompleta, al día y completa; seguimiento persistente de próximos episodios independiente de futuras temporadas. La API devuelve disponibilidad y contadores.
- Dashboard adaptable con películas, series y episodios, descargas, indexadores y cola. Middle utiliza naranja en controles y azul en indicadores de monitorización.
- Ajustes organizados en nueve filas: apariencia/apoyo/diagnóstico; RSS/qBittorrent/políticas de compartición; TMDB/TVDB/automatización; importación y nombres; rutas; biblioteca; mantenimiento/API; copias; contraseña.
- Filtros combinables de descargas por tracker, categoría, etiqueta, estado y texto.
- Importación web de YAML: validación, límite de 512 KiB, escritura segura, reemplazo confirmado y credenciales existentes preservadas. Nuevas definiciones no se activan automáticamente.
- Cardigann `:contains` aplica condiciones de texto; selectores inválidos no se amplían eliminando restricciones. Escapes CSS numéricos y combinaciones relativas de `:has` con `:contains` no se interpretan.
- Puntuación coherente de películas descubiertas e importadas, cutoff y margen de mejora respetados en RSS.
- Importación hardlink/copia/mover, renombrado, registro de transferencias, reseed y políticas de seeding. No se debe mover un archivo todavía compartido sin considerar las consecuencias para el torrent.
- Seis idiomas: español, inglés, francés, alemán, italiano y portugués de Portugal.

## SMB y permisos

Conexiones opcionales SMB nativas de Windows: credenciales DPAPI de máquina, recursos compartidos en el selector, rutas UNC y restauración al iniciar la aplicación o servicio. Independiente de la marca del NAS y de Cinetta. No asigna letras, no fuerza desconexiones y no habilita SMB1.

Sin configurar SMB se mantienen accesos, permisos y rutas existentes; el selector sí abre una vista de discos y recursos. El servicio no hereda las unidades conectadas en otra sesión de Windows. La cuenta del NAS conserva sus permisos: no se exige ni se configura automáticamente una cuenta de solo lectura. Linux/Docker utiliza montajes SMB del sistema operativo.

Los recursos guardados siguen visibles sin conexión; abrirlos puede devolver un error. Un NAS desconectado no bloquea indefinidamente el arranque. Cambiar de máquina requiere reintroducir las credenciales. La API no devuelve contraseñas ni sus blobs cifrados. Véase [SMB](Oberiz_SMB.md).

## Integraciones

Public API v1 con claves de API, idempotencia, perfiles, temporadas seleccionadas y monitorización. Cinetta debe consumir disponibilidad real y separar próximos episodios de futuras temporadas; los cambios de su interfaz pertenecen a su repositorio, no al paquete de Oberiz. Compatibilidad Overseerr opcional.

qBittorrent mantiene categorías y estados existentes durante el escaneo. Las correspondencias siguen siendo necesarias cuando comunica rutas distintas a las accesibles por Oberiz.

## Verificación y límites

87 pruebas unitarias y una prueba de migración correctas; prueba real de almacenamiento optativa excluida de CI. Clippy, formato, compilación frontend, filtros y cobertura de idiomas verificados.

El ciclo RSS → descarga simulada → importación → reescaneo → mejora se prueba con SQLite temporal, RSS HTTP local y cliente simulado, incluyendo errores y recuperación. Se verificaron cifrado DPAPI, rechazo de datos corruptos, respuestas sin secretos, listado sin abrir el NAS y eliminación de credenciales guardadas.

Una prueba optativa del código real de mover entre dos recursos SMB verificó el contenido y eliminó los archivos temporales sin modificar contenido existente. Se ejecutó desde una sesión interactiva: no prueba los permisos de LocalSystem. La autenticación real del NAS desde el servicio necesita configuración de credenciales y verificación operativa. No se han realizado descargas reales de contenido como parte de las pruebas automatizadas.

Las dependencias necesarias para compilar permanecen bloqueadas en Cargo.lock/package-lock.json. Dependabot conserva su configuración, con PR de actualizaciones de versión pausadas; no se eliminan protecciones ni dependencias funcionales.
