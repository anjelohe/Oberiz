# Informes de faltantes — contrato acordado, 2026-10-16

Referencia del productor: proyecto hermano Cinetta,
`docs/cinetta-oberiz-missing-contract-2026-10-16.md`.

Estado: implementacion local, no publicada ni instalada sobre produccion.
Pendiente de prueba con credenciales y biblioteca reales de Cinetta.

## Acceso

- Cinetta ofrece `GET /api/v1/library/inventory` con Bearer exclusivo de lectura,
  instantaneas, cursor y filtros. No acceso al NAS, escaneo ni mutacion por consulta.
- Oberiz ofrece GET `/api/v1/library/missing` y POST
  `/api/v1/library/missing/refresh`; GET nunca inicia trabajos.
- La integracion requiere X-Api-Key y permiso explicito
  `api.missing_reports_enabled=true`, desactivado por defecto incluso al actualizar.
  Sin permiso: 403 antes de consultar proveedores o crear trabajos. No se modifica
  la autorizacion de PUT de inventario ni de peticiones existentes.
- El panel de Oberiz usa sesion administrativa; no necesita activar permiso externo.
- Idempotencia duradera en POST y consultas por job_id opaco estable. Al paginar,
  mantener todos los filtros y el job_id, nunca pasar al ultimo informe de otro trabajo.
- GET sin informe previo devuelve 409 report_not_found (no 404, reservado a ruta
  sin soporte o job_id inexistente). Cinetta lo presenta como «Actualizar para crearlo».

## Semantica de resultados

Solo episodios emitidos y peliculas estrenadas, nunca futuros o sin fecha confirmada.
Metadatos TMDB en es-ES; numeracion de emision estandar y temporada 0 para especiales.
TVDB ID se resuelve mediante /find solo con coincidencia unica, sin comparacion por
titulo. Fechas air_date y release_date validas <= hoy UTC; sin filtro de pais ni
garantia de disponibilidad digital. Una pelicula estrenada en cines puede figurar.
Los informes se guardan aparte: ninguna alta de titulo, monitorizacion, solicitud,
descarga, renombrado ni borrado de multimedia.

El inventario actual de Cinetta siempre declara complete:false: los candidatos
ausentes son provisionales, con state=partial, warning inventory_incomplete y
availability=unknown, incluso si el resultado es cero. No afirmar «no falta nada».
Contenido indexado offline o unknown cuenta como presente para evitar falsos faltantes.
Un archivo con varios episodios (S01E01E02) puede declarar `episode_number_end`
(opcional, como mucho 50 por encima de `episode_number`): cuenta todo el rango como
presente. Sin ese campo, solo cuenta `episode_number` y el resto del rango aparece
como faltante.

Mientras un trabajo esta en cola o en curso, el GET sin job_id devuelve el ultimo
informe terminado de ese filtro; el marcador vacio del trabajo solo se devuelve si
no hay ninguno terminado. Los informes son una cache: con el almacen lleno se
descartan los terminados mas antiguos en vez de rechazar nuevas comprobaciones.
Completar correctamente la comparacion exige tratar tambien errores de metadatos,
numeraciones incompatibles e identificadores desconocidos, sin coincidencias por titulo.

## Series grandes y paginacion

Maximo 100 fragmentos/pagina, 500 items/fragmento y 1 MiB/respuesta. Respetar siempre
el limite mas restrictivo. Dividir series por temporada y despues por fragmentos;
repetir identificadores y titular con temporada/parte. Colecciones: ordenar por fecha
e ID y fragmentar igual. Orden y paginas inmutables por job_id, sin omitir ni repetir
items. Totales de todo el informe: grupos logicos distintos, episodios/peliculas unicos;
no contar fragmentos como series diferentes. Exceso individual: partial y advertencia
report_item_too_large, nunca truncamiento silencioso.

## Acceso y prueba local

Peliculas > Colecciones incompletas; Series > Episodios faltantes; accion individual
en cada tarjeta. Se puede elegir serie/coleccion por ID o todo el inventario.
Abrir no consulta. Ultimo informe hace GET; Actualizar hace POST con Idempotency-Key.
Dejar de esperar cancela solo la espera del navegador, no el trabajo del servidor.

Ajustes > Informes de contenido faltante: URL base de Cinetta, clave library:read
creada en Cinetta y permiso externo opcional. Guardar y configurar TMDB. El panel
interno no requiere activar API publica ni permiso externo; el boton de Cinetta si.
La clave de Cinetta se guarda en la DB privada como las demas credenciales de
conexiones; no se devuelve al navegador. Proteger DB/copias con ACL; no hay cifrado
integral nuevo. Las rutas internas son /api/library/missing y /refresh; el lookup
/api/library/missing/collection?movie_tmdb_id=ID resuelve colecciones sin altas.

Trabajos e idempotencia en missing_reports y missing_report_actions (migracion 0023).
Dos trabajos activos maximo; mismo alcance activo se fusiona. Reinicio marca
interrumpidos como error sin reiniciar consultas; conserva el ultimo informe no
fallido. Timeout 15s/peticion y 20min/trabajo. Sin redirects ni desactivar TLS.
Solo persiste el informe, no el inventario completo de Cinetta.

Limites: 32 MiB de inventario, 500000 entradas, 1000 series, 200 temporadas/serie,
2000 peliculas para descubrir colecciones y 20000 items/informe. Limites de
titulos/temporadas/items generan partial con advertencia. Errores de metadatos por
titulo/temporada conservan resultados parciales. No se usa ETag aun. Un cursor
caducado o cambio de instantanea falla sin mezclar paginas.

Iniciar copia aislada: `./scripts/start-missing-preview.ps1 -Build` (PowerShell 7).
Abrir http://127.0.0.1:2033. Datos/config independientes en
backend/backups/missing-preview, ignorados por Git. No toca el servicio existente.
Crear contrasena de prueba y configurar Cinetta/TMDB. No necesita importar titulos
a Oberiz para consultar inventario. Detener solo el PID devuelto por el script.

Pruebas automaticas: HTTP local simulado de inventario/TMDB y SQLite en memoria;
fechas futuras/desconocidas, especiales, offline, series grandes, idempotencia,
permisos y ausencia de mutaciones de biblioteca/solicitudes/descargas. Frontend:
acciones explicitas, paginacion fija, errores conservan informe previo. No sustituyen
la prueba conjunta con bibliotecas reales ni certifican numeraciones DVD/absolutas.

Referencias TMDB: [temporadas](https://developer.themoviedb.org/reference/tv-season-details),
[colecciones](https://developer.themoviedb.org/reference/collection-details) y
[IDs externos](https://developer.themoviedb.org/reference/find-by-id).
