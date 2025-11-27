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

// Elementos del DOM
let crearSalaForm;
let salasContainer;
let refreshBtn;

// Esperar a que el DOM esté listo
function inicializar() {
    console.log('📄 DOM cargado, inicializando...');

    crearSalaForm = document.getElementById('crear-sala-form');
    salasContainer = document.getElementById('salas-container');
    refreshBtn = document.getElementById('refresh-btn');

    if (!crearSalaForm) {
        console.error('⚠️ No se encontró el formulario #crear-sala-form');
        return;
    }

    if (!salasContainer) {
        console.error('⚠️ No se encontró el contenedor #salas-container');
        return;
    }

    if (!refreshBtn) {
        console.error('⚠️ No se encontró el botón #refresh-btn');
        return;
    }

    console.log('✅ Elementos DOM encontrados');

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

    // Cargar salas iniciales
    console.log('📥 Cargando salas iniciales...');
    cargarSalas();

    // Obtener y mostrar ruta del log
    obtenerRutaLog();

    mostrarNotificacion('🟢 Aplicación lista', 'success');
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
