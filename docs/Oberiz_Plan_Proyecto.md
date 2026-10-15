# Oberiz — Plan de producto

## Base v1.0.2

Biblioteca de películas y series, automatización/RSS, perfiles, qBittorrent,
indexadores, API y seis idiomas. Inventario Cinetta o escaneo local de solo
lectura, sin importaciones directas ni conexiones SMB gestionadas por Oberiz.
Renombrado remoto opcional desactivado por defecto, diario persistente y
recuperación detenida. Véanse la [memoria](Oberiz_Memoria_Actualizada.md) y la
[guía de almacenamiento](Oberiz_Almacenamiento_Simplificado.md).

## Principios

1. Automatización fiable y decisiones basadas en archivos realmente existentes.
2. No borrar, pausar ni cancelar torrents por cambios externos implícitos.
3. Proteger configuración, credenciales, datos y bibliotecas en actualizaciones.
4. Separar inventario de operaciones: recibir rutas no concede acceso a los archivos.
5. Mantener las credenciales fuera de Git y de respuestas de API.

## Próximas verificaciones operativas

- Instalar conservando datos y comprobar la copia previa a la migración 0022.
- Configurar el envío de inventario en la instancia instalada de Cinetta o las carpetas locales de solo lectura.
- Verificar qBittorrent remoto y su consulta de directorios antes de activar renombrado; no usar organizadores concurrentes sobre los mismos destinos.
- La prueba aislada con qBittorrent 5.2.3 ya verifica renombrado, exportación/hash, recuperación detenida, recheck y protección de un destino ocupado. No sustituye las comprobaciones de cada instalación.
- Recuperación de registros de importación antiguos y torrents v2 puros: manual por ahora.
- Seleccionar contenido y límites antes de pruebas reales de adquisición.
- Validar paquetes Linux/Docker/Windows en instalaciones reales y mantener pruebas de recuperación y migración.

La publicación de código no configura automáticamente credenciales ni cambia permisos del NAS. Cambiar el número de versión del producto no justifica borrar datos del usuario.
