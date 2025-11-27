const { invoke } = window.__TAURI__.core;

// Elementos del DOM
const crearSalaForm = document.getElementById('crear-sala-form');
const salasContainer = document.getElementById('salas-container');
const refreshBtn = document.getElementById('refresh-btn');

// Cargar salas al iniciar
document.addEventListener('DOMContentLoaded', () => {
    cargarSalas();
});

// Evento para crear sala
crearSalaForm.addEventListener('submit', async (e) => {
    e.preventDefault();

    const formData = new FormData(crearSalaForm);
    const request = {
        nombre: formData.get('nombre'),
        capacidad: parseInt(formData.get('capacidad'))
    };

    try {
        await invoke('crear_sala', { request });
        crearSalaForm.reset();
        await cargarSalas();
        mostrarNotificacion('✅ Sala creada exitosamente', 'success');
    } catch (error) {
        mostrarNotificacion(`❌ Error: ${error}`, 'error');
    }
});

// Evento para refrescar
refreshBtn.addEventListener('click', cargarSalas);

// Función para cargar salas
async function cargarSalas() {
    salasContainer.innerHTML = '<div class="loading">⏳ Cargando salas...</div>';

    try {
        const salas = await invoke('listar_salas');

        if (salas.length === 0) {
            salasContainer.innerHTML = '<div class="empty">📭 No hay salas registradas</div>';
            return;
        }

        salasContainer.innerHTML = salas.map(sala => crearTarjetaSala(sala)).join('');

        // Agregar eventos a los botones
        document.querySelectorAll('[data-action]').forEach(btn => {
            btn.addEventListener('click', manejarAccionSala);
        });
    } catch (error) {
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
                <h3 class="sala-nombre">${sala.nombre}</h3>
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
        mostrarNotificacion(`❌ Error: ${error}`, 'error');
    }
}

// Función para mostrar notificaciones
function mostrarNotificacion(mensaje, tipo) {
    // Crear elemento de notificación
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

    // Remover después de 3 segundos
    setTimeout(() => {
        notif.style.animation = 'slideOut 0.3s ease';
        setTimeout(() => notif.remove(), 300);
    }, 3000);
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
