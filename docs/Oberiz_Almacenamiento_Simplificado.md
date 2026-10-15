# Biblioteca, renombrado y recuperación — 15 de octubre de 2026

Incluido en v1.0.2. Publicar esta versión no actualiza automáticamente la instancia
instalada ni modifica el NAS o los torrents existentes: hay que instalar la nueva versión.

## Responsabilidades

- qBittorrent descarga en el destino de su categoría. Oberiz ya no importa,
  copia, crea hardlinks, mueve archivos ni construye carpetas de reseed.
- Cinetta envía su inventario a `PUT /api/v1/library`, con API pública y clave
  habilitadas. Seleccionar Cinetta en Oberiz no activa el emisor de Cinetta.
- Alternativamente se seleccionan carpetas accesibles localmente y se escanean
  en solo lectura. No hay configuración de SMB, credenciales ni traducción de
  rutas de descarga. Un montaje gestionado por el sistema operativo es externo
  a Oberiz; sus permisos no se cambian.
- El origen del inventario y el renombrado son ajustes independientes.

## Configuración

1. Configurar las categorías y destinos en qBittorrent.
2. En **Inventario de la biblioteca**, elegir **Cinetta** o **Carpetas locales**.
3. Con Cinetta: activar la API pública v1, generar una clave y guardar; configurar
   URL/clave de Oberiz y activar el inventario en los ajustes de Cinetta.
   Se muestra la fecha del último informe aceptado. Las adquisiciones continúan
   aunque Cinetta no informe. La casilla opcional de pausa, desactivada por defecto
   también al actualizar, permite retener nuevas adquisiciones si falta el informe
   o tiene más de 24 horas. Se conserva el último inventario sin escanear rutas remotas.
4. Con carpetas locales: añadir una o varias carpetas, guardar y ejecutar o
   programar el escaneo. Escanear no mueve, renombra ni borra archivos.
5. En **Renombrado mediante qBittorrent y recuperación**, el renombrado está
   **desactivado por defecto**, también al actualizar una instalación antigua.
   Su activación exige el nuevo campo `qbittorrent_rename_enabled`; el antiguo
   `rename_enabled` enviado por clientes anteriores no activa operaciones remotas.
   Afecta a los trabajos de descarga todavía pendientes de procesar: no renombra
   retroactivamente toda la biblioteca ni los trabajos ya completados.

## Renombrado opcional

Solo se procesan descargas asociadas a trabajos de Oberiz y terminadas. Se
obtiene la lista por `GET /api/v2/torrents/files`, se calcula un plan completo y
se guarda antes de cualquier cambio. Se conserva la extensión y el directorio
actual: no se usa `setLocation`, ni se divide un torrent entre discos.

Se utiliza `POST /api/v2/torrents/renameFile` (no el nombre visible del torrent).
Cada paso se verifica consultando otra vez sus archivos. Una respuesta HTTP 200
no se considera suficiente. Los reintentos reconocen pasos ya aplicados tras
una desconexión; no repiten el renombrado ni borran el diario.

Un torrent de película con varios vídeos, episodios no identificables, subtítulos
externos, rutas inválidas o destinos duplicados necesita revisión manual. No se
renombran parcialmente esos planes. Si falla un paso de un plan válido después
de haber aplicado otros, se conservan el diario y los cambios confirmados; no se
intenta un rollback de archivos a ciegas.

Además de comprobar el inventario del torrent, Oberiz consulta el directorio de
destino mediante `app/getDirectoryContent` en qBittorrent, antes del primer cambio
y de nuevo antes de cada paso. Si la consulta falla o la versión no la admite,
se bloquea el renombrado. No requiere SMB: la lectura de nombres la hace qBittorrent.
Un archivo destino ocupado bloquea el cambio. En recuperación solo se admite
reenlazar un destino ya existente cuando no existe a la vez el archivo origen.

La prueba real descubrió que `renameFile` de qBittorrent 5.2.3 puede sobrescribir
un archivo ajeno sin devolver conflicto HTTP; por eso no basta con comprobar la
lista interna del torrent ni con confiar en un 409. La protección añadida se ha
verificado contra un archivo señuelo real y con el código Rust de Oberiz.

La API no ofrece una operación atómica «renombrar sin reemplazar». No se debe
compartir un destino entre torrents que gestionan los mismos archivos ni ejecutar
otros organizadores sobre esas rutas simultáneamente. Cambios externos entre la
comprobación y el renombrado o entradas ocultas no expuestas por la API son límites
del diseño sin acceso al sistema de archivos. No se promete detección universal.

## Archivo y recuperación

Por defecto se conserva el `.torrent` original, su hash, destino en qBittorrent,
índices/tamaños y correspondencias de nombres en `torrent_archives`, dentro de
SQLite. Las copias normales de la base de datos incluyen estos datos; no hacen
falta copias de vídeos ni acceso SMB. Las claves privadas del tracker contenidas
en un `.torrent` también son secretos: proteger las copias de seguridad.

El renombrado necesita ese archivo aunque se desactive la conservación opcional.
Este archivo no sustituye una copia de la sesión/configuración completa de
qBittorrent: no conserva estadísticas, colas ni todas sus preferencias por torrent.
Antes de reanudar una recuperación hay que revisar también la selección y
prioridades de archivos, categorías y límites en qBittorrent.
Si la exportación o su validación fallan, no empieza el renombrado ni se da por
completado el trabajo para la limpieza automática.

**Descargas y recuperación → Recuperar detenido** requiere confirmación y:

- Rechaza modificar un torrent ya presente en qBittorrent.
- Comprueba la identidad del `.torrent` (recuperación v1/híbrida; v2 pura requiere
  recuperación manual por ahora).
- Lo añade con `paused=true` y `stopped=true`, gestión automática desactivada,
  sin omitir la comprobación y conservando la disposición original.
- Aplica los nombres registrados a través de qBittorrent y solicita un recheck.
- No llama a start/resume. El usuario revisa y reanuda en qBittorrent.
- No aplica limpieza de compartición a una recuperación detenida. Solo vuelve
  a evaluarla tras observar que el usuario la ha reanudado para compartir.

Los registros de importación antiguos y sus `.torrent` externos se conservan,
pero no se convierten automáticamente en recuperaciones remotas: falta una
correspondencia fiable entre el antiguo destino local y el del cliente. La UI
no ofrece el antiguo reseed que copiaba archivos; esos casos se recuperan
manualmente en qBittorrent. Los registros con archivo remoto no se eliminan
desde la limpieza de registros obsoletos.

## Actualización segura

La migración 0022 conserva historial, archivos indexados, carpetas y ajustes
antiguos necesarios para compatibilidad/rollback. El arranque crea primero una
copia SQLite coherente en `data/backups/oberiz-before-storage-simplification-*.sqlite3`
si ya existe una base anterior. Si la copia falla, no se aplica la transición.
Después se elimina únicamente la antigua configuración de credenciales SMB;
no se desconectan montajes del sistema operativo ni se toca el NAS.

Las políticas siguen siendo una función separada. Las políticas antiguas de
«quitar torrent y original» se convierten en «quitar torrent conservando archivos».
El borrado manual desde Descargas permite eliminar también los archivos mediante
una casilla desmarcada por defecto y una confirmación con aviso permanente.
qBittorrent ejecuta esa orden incluso sin acceso de Oberiz al NAS. Las políticas
automáticas y el rechazo de releases siguen conservando los archivos.

Los campos y el endpoint de sugerencias de correspondencias antiguos se mantienen
temporalmente por compatibilidad; este último devuelve una lista vacía y no
consulta discos ni qBittorrent. No hay configuración ni uso de traducción de
rutas en el procesamiento nuevo de descargas.

## Verificación y límites

Pruebas con SQLite de prueba, archivos sintéticos, clientes simulados y HTTP
loopback. No se ha accedido al NAS ni se han tocado torrents del usuario.
El emisor JavaScript existente de Cinetta se ha revisado y sus pruebas pasan;
no se modifica Cinetta ni se confirma que el envío esté habilitado en la instancia
instalada del usuario.

También se ha probado qBittorrent real **v5.2.3**, en un perfil aislado, con WebUI
solo loopback, credenciales efímeras y un torrent privado de 8 KiB sin trackers:
renombrado, exportación con hash conservado, retirada sin borrar datos, recuperación
detenida, recheck al 100 % y rechazo de una colisión mediante el guard real de Rust.
Prueba reproducible: `node scripts/test-qbittorrent-isolated.mjs <ruta-absoluta-qbittorrent>`.
Evidencia privada: `backend/backups/storage-simplification-20261015/real-qbt-9RfBU6/result.json`.
El proceso aislado se cerró al terminar. La integración con la instalación del NAS
y la activación de inventario en Cinetta se dejan a la configuración del usuario.

Referencias: [API oficial qBittorrent](https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-5.0)),
[implementación oficial de los controladores](https://github.com/qbittorrent/qBittorrent/blob/master/src/webui/api/torrentscontroller.cpp).
