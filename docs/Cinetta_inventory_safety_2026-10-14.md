# Inventario Cinetta: correcciones de seguridad

## Alcance

Solo inventario informativo. Oberiz recibe rutas y metadatos para conocer la
disponibilidad, sin ordenar a Cinetta que mueva, renombre o borre archivos.
Seleccionar import.method=external en Almacenamiento; no se cambia ninguna
configuracion existente automaticamente. Reiniciar/recompilar para aplicar
las fuentes corregidas. No se despliega ni toca la base de datos real.

## Correcciones

- Autorizacion de API antes de leer JSON. Dos informes simultaneos como maximo,
  sin cola de cuerpos; 429 permite reintento. Cuerpo limitado a 128 MiB, un millon
  de entradas y 1024 raices; validacion de rutas, rangos y medidas sin overflow.
- External bloquea reseed tambien para trabajos antiguos, descubrimiento,
  escaneos manuales/programados, comprobacion de carpetas y sugerencias NAS.
  La UI no ofrece reseed cuando external o cuando no puede comprobar ajustes.
- Importaciones, escaneos y reseed comparten una barrera con cambios de modo.
  Cambiar a external espera a operaciones activas; no las cancela a medias.
  La eliminacion destructiva de torrents usa tambien esa barrera; limpieza
  de trabajos antiguos en external no solicita borrado de datos.
- Los informes son aditivos por defecto. Una omision no declara ausencia.
  allow_large_removal=true es confirmacion explicita para cualquier omision,
  no solo grandes bajas, y requiere raiz disponible y no repentinamente vacia.
  Sin raices, fuera de ellas o ante incertidumbre se conserva disponibilidad.
- Reclasificar un episodio como pelicula limpia enlaces y recalcula episodios.
  Declarar simultaneamente identidades incompatibles para una ruta se rechaza.
- External devuelve superseded vacio y no infiere propietarios de descargas.
  En modo gestionado, vinculacion requiere ruta fuente completa/mapeada, no
  basename; reemplazo parcial no nomina archivos de episodios multiples.

## Validacion y limites

Integracion con el snapshot final de la otra sesion: 111 pruebas unitarias y
una de migracion correctas (112 en total), una prueba real omitida. Web compilada
y cuatro comprobaciones frontend correctas. Se conserva la validacion estricta
de episodios positivos y rangos sin overflow introducida por la otra sesion.
Instaladores/binarios de v1.0.0 ya publicados no se modifican automaticamente.

Pruebas con SQLite en memoria, servidor HTTP loopback y archivos temporales
sinteticos. Nunca ejecutar live_storage_move_probe en la validacion automatica.
No NAS, proveedor, qBittorrent real ni biblioteca real. No habilitar limpieza
ni confirmacion de bajas automaticamente. Un solo cliente emisor y envios
serializados: no se implementa protocolo multi-cliente/versiones de snapshot.
Pendiente prueba conjunta con Cinetta antes de activar envios en produccion.

Respaldo y manifiesto SHA-256 en el repositorio Cinetta:
backups/oberiz-fixes-20261014-175230/baseline y MANIFEST.json.
La transferencia de fuentes debe comprobar primero que el destino no ha
cambiado desde el respaldo y comprobar despues los hashes de los archivos.
