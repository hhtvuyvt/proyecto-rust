# Rust Bookstore API

Este proyecto es el backend de una tienda de libros escrita en **Rust**. Proporciona una API REST construída con **Axum** y **SQLx** para gestionar usuarios (registro, actualización, eliminación) y otros recursos que iremos ampliando (catálogo de libros, pedidos, reseñas, etc.). La aplicación está pensada como base para una futura página web completa de e-commerce de libros.

## Tabla de contenidos

1. [Características principales](#características-principales)
2. [Stack tecnológico](#stack-tecnológico)
3. [Arquitectura a alto nivel](#arquitectura-a-alto-nivel)
4. [Requisitos previos](#requisitos-previos)
5. [Configuración rápida](#configuración-rápida)
6. [Comandos disponibles](#comandos-disponibles)
7. [Endpoints actuales](#endpoints-actuales)
8. [Pruebas](#pruebas)
9. [Próximos pasos](#próximos-pasos)
10. [Contribuciones](#contribuciones)

## Características principales

- **API REST** en Rust para gestionar usuarios (creación, consulta, actualización y borrado).
- **Validaciones** de entrada para asegurar nombres y correos electrónicos consistentes.
- **Persistencia** con SQLite usando **SQLx** y migraciones automatizadas.
- **Diseño modular**: separación clara entre rutas, controladores y modelos.
- **Pruebas de integración** con Axum y Tokio para garantizar el comportamiento de los endpoints.

## Stack tecnológico

- **Rust**: lenguaje principal.
- **Axum**: framework web asíncrono.
- **SQLx**: acceso a base de datos asíncrono.
- **Tokio**: runtime asíncrono base.
- **tower**: capas y utilidades para servicios.
- **Serde**: serialización y deserialización.
- **SQLite**: base de datos predeterminada (fácil de iniciar y portable).

## Arquitectura a alto nivel

```text
┌─────────────┐      HTTP       ┌─────────────┐        ┌─────────────┐
│    Cliente   │ ─────────────▶ │  Axum Router │──SQL──▶│  SQLite DB  │
└─────────────┘                 └─────────────┘        └─────────────┘
        │                             │
        │                             ▼
        │                   Handlers / Controladores
        │                             │
        │                             ▼
        │                    Modelos + Lógica de negocio
        │
        └── Futuras extensiones: servicio web, SPA, mobile, etc.
```

- `src/main.rs`: punto de entrada. Carga configuración, ejecuta migraciones y levanta el servidor.
- `src/routes`: define los endpoints y agrupa routers temáticos (`/users`, `/health`, etc.).
- `src/handlers`: contiene la lógica para cada endpoint (crear usuario, validar campos, etc.).
- `src/models`: define estructuras de datos, validaciones y errores de negocio.
- `tests/`: pruebas de integración que ejercitan la API completa.

## Requisitos previos

- **Rust** (recomendado: `rustup` + toolchain estable actual).
- **cargo** incluido con Rust.
- **SQLite** (opcional, se puede usar la versión embebida).
- (Opcional) `sqlx-cli` para correr migraciones manuales: `cargo install sqlx-cli`.

## Configuración rápida

1. **Clonar el repositorio**
   ```bash
   git clone <url-del-repo>
   cd proyecto-rust
   ```
2. **Configurar variables de entorno**
   Crea un archivo `.env` (puedes basarte en `.env.example` si lo añades) con al menos:
   ```env
   DATABASE_URL=sqlite://proyecto.db
   HOST=127.0.0.1
   PORT=3000
   ```
3. **Ejecutar migraciones**

   ```bash
   cargo sqlx migrate run
   ```

   Si prefieres que la app lo haga automáticamente, basta con iniciar el servidor; `main.rs` ejecuta las migraciones al arrancar.

4. **Iniciar la API**
   ```bash
   cargo run
   ```
   La API quedará escuchando en `http://127.0.0.1:3000`.

## Comandos disponibles

- `cargo run`: compila y levanta el servidor.
- `cargo test`: ejecuta pruebas unitarias e integrales.
- `cargo fmt`: formatea el código según `rustfmt`.
- `cargo clippy`: revisa el código con lints adicionales.
- `cargo sqlx migrate run`: aplica migraciones a la base de datos.

## Endpoints actuales

| Método | Ruta         | Descripción                             |
| ------ | ------------ | --------------------------------------- |
| GET    | `/health`    | Devuelve estado saludable del servicio. |
| GET    | `/users`     | Lista usuarios registrados.             |
| GET    | `/users/:id` | Recupera un usuario por `id`.           |
| POST   | `/users`     | Crea un nuevo usuario.                  |
| PATCH  | `/users/:id` | Actualiza nombre/email de un usuario.   |
| DELETE | `/users/:id` | Elimina un usuario existente.           |

_Todas las respuestas y errores se devuelven en JSON. Las entradas se validan y devuelven mensajes descriptivos en caso de datos incorrectos._

## Pruebas

Ejecuta `cargo test` para correr la suite de pruebas de integración. Estas pruebas levantan un `Router` en memoria, simulan requests HTTP y verifican respuestas y efectos de base de datos.

Próximamente se agregarán casos negativos (por ejemplo, creación con email inválido) y nuevas suites para catálogo de libros y pedidos.

## Próximos pasos

1. Añadir endpoints para gestionar el catálogo de libros (`/books`), inventario y pedidos.
2. Integrar autenticación/autorización.
3. Desplegar frontend (SPA en Rust + WebAssembly o un framework web tradicional).
4. Configurar CI/CD y despliegue a un entorno controlado.
5. Documentar colección de Postman/Bruno/Insomnia para pruebas manuales.

## Contribuciones

1. Crea un fork y una rama descriptiva (`feature/add-books-endpoint`).
2. Asegúrate de correr `cargo fmt`, `cargo clippy` y `cargo test` antes de abrir el PR.
3. Explica claramente los cambios y añade pruebas cuando sea posible.

¡Gracias por interesarte en este proyecto! Si tienes dudas o sugerencias, abre un issue o contacta al autor.
