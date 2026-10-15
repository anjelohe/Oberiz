# Oberiz — Memoria del proyecto

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
