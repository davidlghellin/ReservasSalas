// Esperar a que Tauri esté listo
let invoke;

// Detectar la versión de Tauri y obtener invoke
if (window.__TAURI__) {
    // Tauri v2
    invoke = window.__TAURI__.core.invoke;
} else if (window.__TAURI_INTERNALS__) {
    // Tauri v2 alternativo
    invoke = window.__TAURI_INTERNALS__.invoke;
} else {
    console.error('❌ Tauri API no disponible');
    // Función de fallback para debugging
    invoke = async (cmd, args) => {
        console.error(`No se puede invocar ${cmd} - Tauri no inicializado`);
        throw new Error('Tauri API no disponible');
    };
}

console.log('🚀 JavaScript cargado, invoke:', typeof invoke);

// Estado de autenticación
let usuarioActual = null;
let tokenActual = null;

// Elementos del DOM
let loginScreen;
let mainScreen;
let loginForm;
let loginError;
let loginSubmitBtn;
let crearSalaForm;
let salasContainer;
let refreshBtn;
let logoutBtn;
let userNameEl;
let userEmailEl;
let userRolEl;

// Esperar a que el DOM esté listo
function inicializar() {
    console.log('📄 DOM cargado, inicializando...');

    // Elementos de login
    loginScreen = document.getElementById('login-screen');
    mainScreen = document.getElementById('main-screen');
    loginForm = document.getElementById('login-form');
    loginError = document.getElementById('login-error');
    loginSubmitBtn = document.getElementById('login-submit-btn');

    // Elementos de la pantalla principal
    crearSalaForm = document.getElementById('crear-sala-form');
    salasContainer = document.getElementById('salas-container');
    refreshBtn = document.getElementById('refresh-btn');
    logoutBtn = document.getElementById('logout-btn');
    userNameEl = document.getElementById('user-name');
    userEmailEl = document.getElementById('user-email');
    userRolEl = document.getElementById('user-rol');

    // Configurar eventos de login
    if (loginForm) {
        loginForm.addEventListener('submit', manejarLogin);
    }

    // Configurar logout
    if (logoutBtn) {
        logoutBtn.addEventListener('click', manejarLogout);
    }

    // Mostrar pantalla de login inicialmente
    mostrarPantallaLogin();

    // Inicializar eventos de la pantalla principal (se ejecutará después del login)
    inicializarPantallaPrincipal();
}

// Función para manejar el login
async function manejarLogin(e) {
    e.preventDefault();
    console.log('🔐 Intentando login...');

    const email = document.getElementById('login-email').value;
    const password = document.getElementById('login-password').value;

    if (!email || !password) {
        mostrarErrorLogin('Email y contraseña son requeridos');
        return;
    }

    loginSubmitBtn.disabled = true;
    loginSubmitBtn.textContent = '⏳ Iniciando sesión...';
    ocultarErrorLogin();

    try {
        const response = await invoke('login_usuario', {
            request: { email, password }
        });

        console.log('✅ Login exitoso:', response);
        usuarioActual = response.usuario;
        tokenActual = response.token;

        mostrarPantallaPrincipal();
        await cargarSalas();
        mostrarNotificacion('✅ Login exitoso', 'success');

        // Obtener y mostrar ruta del log
        obtenerRutaLog();
    } catch (error) {
        console.error('❌ Error en login:', error);
        mostrarErrorLogin(`Error: ${error}`);
    } finally {
        loginSubmitBtn.disabled = false;
        loginSubmitBtn.textContent = '🚀 Iniciar Sesión';
    }
}

// Función para manejar el logout
async function manejarLogout() {
    console.log('🚪 Cerrando sesión...');
    try {
        await invoke('logout_usuario');
        usuarioActual = null;
        tokenActual = null;
        mostrarPantallaLogin();
        mostrarNotificacion('👋 Sesión cerrada', 'info');
    } catch (error) {
        console.error('Error en logout:', error);
        // Aún así limpiar el estado local
        usuarioActual = null;
        tokenActual = null;
        mostrarPantallaLogin();
    }
}

// Función para mostrar pantalla de login
function mostrarPantallaLogin() {
    if (loginScreen) loginScreen.style.display = 'flex';
    if (mainScreen) mainScreen.style.display = 'none';
    if (loginForm) loginForm.reset();
    ocultarErrorLogin();
}

// Función para mostrar pantalla principal
function mostrarPantallaPrincipal() {
    if (loginScreen) loginScreen.style.display = 'none';
    if (mainScreen) mainScreen.style.display = 'block';

    if (usuarioActual) {
        if (userNameEl) userNameEl.textContent = `👤 ${usuarioActual.nombre}`;
        if (userEmailEl) userEmailEl.textContent = `📧 ${usuarioActual.email}`;
        if (userRolEl) userRolEl.textContent = `🎫 ${usuarioActual.rol}`;
    }
}

// Función para mostrar error de login
function mostrarErrorLogin(mensaje) {
    if (loginError) {
        loginError.textContent = mensaje;
        loginError.style.display = 'block';
    }
}

// Función para ocultar error de login
function ocultarErrorLogin() {
    if (loginError) {
        loginError.style.display = 'none';
    }
}

// Inicializar eventos de la pantalla principal
function inicializarPantallaPrincipal() {
    if (!crearSalaForm || !salasContainer || !refreshBtn) {
        return;
    }

    // Agregar eventos
    crearSalaForm.addEventListener('submit', async (e) => {
        e.preventDefault();
        console.log('📝 Formulario enviado');

        const formData = new FormData(crearSalaForm);
        const request = {
            nombre: formData.get('nombre'),
            capacidad: parseInt(formData.get('capacidad'))
        };

        console.log('📤 Enviando solicitud:', request);

        try {
            const resultado = await invoke('crear_sala', { request });
            console.log('✅ Sala creada:', resultado);
            crearSalaForm.reset();
            await cargarSalas();
            mostrarNotificacion('✅ Sala creada exitosamente', 'success');
        } catch (error) {
            console.error('❌ Error al crear sala:', error);
            mostrarNotificacion(`❌ Error: ${error}`, 'error');
        }
    });

    refreshBtn.addEventListener('click', () => {
        console.log('🔄 Refrescando salas...');
        cargarSalas();
    });

}

// Función para obtener y mostrar la ruta del log
async function obtenerRutaLog() {
    try {
        const logPath = await invoke('get_log_path');
        console.log(`📋 Logs guardados en: ${logPath}`);

        // Crear banner informativo en la UI
        const banner = document.createElement('div');
        banner.innerHTML = `📋 Logs: <code style="background: rgba(0,0,0,0.1); padding: 2px 6px; border-radius: 4px;">${logPath}</code>`;
        banner.style.cssText = `
            position: fixed;
            bottom: 10px;
            left: 10px;
            padding: 8px 12px;
            background: rgba(255, 255, 255, 0.95);
            border: 1px solid #ddd;
            border-radius: 6px;
            font-size: 0.85rem;
            color: #666;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
            z-index: 1000;
            font-family: monospace;
        `;
        document.body.appendChild(banner);
    } catch (error) {
        console.error('Error obteniendo ruta del log:', error);
    }
}

// Función para cargar salas
async function cargarSalas() {
    console.log('📡 Solicitando lista de salas...');
    salasContainer.innerHTML = '<div class="loading">⏳ Cargando salas...</div>';

    try {
        const salas = await invoke('listar_salas');
        console.log('✅ Salas recibidas:', salas);

        if (!salas || salas.length === 0) {
            salasContainer.innerHTML = '<div class="empty">📭 No hay salas registradas</div>';
            return;
        }

        salasContainer.innerHTML = salas.map(sala => crearTarjetaSala(sala)).join('');

        // Agregar eventos a los botones
        document.querySelectorAll('[data-action]').forEach(btn => {
            btn.addEventListener('click', manejarAccionSala);
        });

        console.log(`✅ ${salas.length} salas renderizadas`);
    } catch (error) {
        console.error('❌ Error al cargar salas:', error);
        salasContainer.innerHTML = `<div class="empty">❌ Error al cargar salas: ${error}</div>`;
    }
}

// Función para crear tarjeta de sala
function crearTarjetaSala(sala) {
    const estadoClass = sala.activa ? 'activa' : 'inactiva';
    const estadoTexto = sala.activa ? '✅ Activa' : '🚫 Inactiva';

    return `
        <div class="sala-card ${estadoClass}">
            <div class="sala-header">
                <h3 class="sala-nombre">${escapeHtml(sala.nombre)}</h3>
                <span class="sala-estado ${estadoClass}">${estadoTexto}</span>
            </div>
            <div class="sala-info">
                <div class="sala-capacidad">
                    <span>👥 Capacidad:</span>
                    <strong>${sala.capacidad} personas</strong>
                </div>
                <div class="sala-id">ID: ${sala.id}</div>
            </div>
            <div class="sala-actions">
                ${sala.activa ? `
                    <button
                        class="btn btn-warning"
                        data-action="desactivar"
                        data-id="${sala.id}"
                    >
                        🚫 Desactivar
                    </button>
                ` : `
                    <button
                        class="btn btn-success"
                        data-action="activar"
                        data-id="${sala.id}"
                    >
                        ✅ Activar
                    </button>
                `}
            </div>
        </div>
    `;
}

// Función para manejar acciones de sala
async function manejarAccionSala(e) {
    const action = e.target.dataset.action;
    const id = e.target.dataset.id;

    console.log(`🎬 Acción: ${action} en sala ${id}`);

    try {
        if (action === 'activar') {
            await invoke('activar_sala', { id });
            mostrarNotificacion('✅ Sala activada', 'success');
        } else if (action === 'desactivar') {
            await invoke('desactivar_sala', { id });
            mostrarNotificacion('🚫 Sala desactivada', 'success');
        }

        await cargarSalas();
    } catch (error) {
        console.error('❌ Error en acción:', error);
        mostrarNotificacion(`❌ Error: ${error}`, 'error');
    }
}

// Función para mostrar notificaciones
function mostrarNotificacion(mensaje, tipo) {
    const notif = document.createElement('div');
    notif.textContent = mensaje;
    notif.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        padding: 15px 25px;
        background: ${tipo === 'success' ? '#28a745' : '#dc3545'};
        color: white;
        border-radius: 8px;
        font-weight: 600;
        box-shadow: 0 4px 12px rgba(0,0,0,0.3);
        z-index: 1000;
        animation: slideIn 0.3s ease;
    `;

    document.body.appendChild(notif);

    setTimeout(() => {
        notif.style.animation = 'slideOut 0.3s ease';
        setTimeout(() => notif.remove(), 300);
    }, 3000);
}

// Función helper para escapar HTML
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Agregar animaciones CSS
const style = document.createElement('style');
style.textContent = `
    @keyframes slideIn {
        from {
            transform: translateX(400px);
            opacity: 0;
        }
        to {
            transform: translateX(0);
            opacity: 1;
        }
    }

    @keyframes slideOut {
        from {
            transform: translateX(0);
            opacity: 1;
        }
        to {
            transform: translateX(400px);
            opacity: 0;
        }
    }
`;
document.head.appendChild(style);

// Inicializar cuando el DOM esté listo
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', inicializar);
} else {
    // El DOM ya está listo
    inicializar();
}
