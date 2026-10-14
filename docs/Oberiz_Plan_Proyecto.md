# Oberiz — Plan de producto

## Base v1.0.0

Primera versión: biblioteca de películas y series, automatización/RSS, perfiles, qBittorrent, indexadores, importación y seeding, escaneo multidisco, API, seis idiomas y conexiones opcionales SMB de Windows. Véase la [memoria](Oberiz_Memoria_Actualizada.md).

## Principios

1. Automatización fiable y decisiones basadas en archivos realmente existentes.
2. No borrar, pausar ni cancelar torrents por cambios externos implícitos.
3. Proteger configuración, credenciales, datos y bibliotecas en actualizaciones.
4. Explicar errores de acceso; no confundir listar un disco con poder importar en él.
5. Mantener las credenciales fuera de Git y de respuestas de API.

## Próximas verificaciones operativas

- Instalar conservando datos y configurar una conexión SMB desde el servicio.
- Verificar enumeración, lectura y una transferencia temporal en ese contexto, con correspondencias de qBittorrent correctas.
- Seleccionar contenido y límites antes de pruebas reales de adquisición.
- Validar paquetes Linux/Docker/Windows en instalaciones reales y mantener pruebas de recuperación y migración.

La publicación de código no configura automáticamente credenciales ni cambia permisos del NAS. Cambiar el número de versión del producto no justifica borrar datos del usuario.
