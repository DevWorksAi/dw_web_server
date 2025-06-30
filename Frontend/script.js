// Alternar menu do usuário
const botaoUsuario = document.getElementById('menu-usuario-btn');
const menuUsuario = document.getElementById('menu-usuario');
botaoUsuario.addEventListener('click', (e) => {
  e.stopPropagation(); // evita fechar imediatamente
  menuUsuario.classList.toggle('open');
});

// Alternar lista de configurações
const btnConfig = document.getElementById('btn-config');
const listaConfig = document.getElementById('lista-config');
btnConfig.addEventListener('click', (e) => {
  e.stopPropagation();
  listaConfig.classList.toggle('open');
});

// Fecha menu se clicar fora
document.addEventListener('click', () => {
  menuUsuario.classList.remove('open');
  listaConfig.classList.remove('open');
});

// WebSocket
let socket;
let nome = "Guest";

document.addEventListener("DOMContentLoaded", function () {
  const input = document.querySelector(".chat-input");
  const sendButton = document.querySelector(".send-button");
  const messagesContainer = document.querySelector(".messages");
  

  // Conectar ao WebSocket
  socket = new WebSocket("ws://localhost:3000/ws");

  socket.onopen = () => {
    console.log("Conectado ao servidor Rust");

    const joinMessage = {
      type: "join_chat",
      username: nome
    };
    socket.send(JSON.stringify(joinMessage));
  };

  socket.onmessage = (event) => {
    const data = JSON.parse(event.data);

    let texto = "";

    if (data.type === "message") {
      texto = `${data.username}: ${data.text}`;
    } else if (data.type === "user_joined") {
      texto = `${data.username} entrou na conversa!`;
    } else if (data.type === "user_left") {
      texto = `${data.username} saiu da conversa.`;
    } else if (data.type === "error") {
      texto = `Erro: ${data.message}`;
    } else {
      texto = `[Tipo desconhecido]: ${event.data}`;
    }

    const msg = document.createElement("div");
    msg.classList.add("mensagem");
    msg.textContent = texto;
    messagesContainer.appendChild(msg);
    messagesContainer.scrollTop = messagesContainer.scrollHeight;
  };

  socket.onerror = (error) => {
    console.error("Erro no WebSocket:", error);
  };

  function enviarMensagem() {
    const texto = input.value.trim();

    if (!texto) return;

    if (socket.readyState === WebSocket.OPEN) {
      const message = {
        type: "send_message",
        text: texto
      };

      socket.send(JSON.stringify(message));
      input.value = "";
    } else {
      alert("⚠️ Conexão com o servidor ainda não foi estabelecida.");
      console.log("Estado do socket:", socket.readyState);
    }
  }

  sendButton.addEventListener("click", enviarMensagem);
  input.addEventListener("keypress", function (e) {
    if (e.key === "Enter") {
      e.preventDefault();
      enviarMensagem();
    }
  });

  // Alterar nome de usuário
  const alterarNome = document.getElementById("alterar-nome");
  alterarNome.addEventListener("click", () => {
    const novoNome = prompt("Digite seu novo nome:");
    if (novoNome) {
      nome = novoNome;
      const joinMessage = {
        type: "join_chat",
        username: nome
      };
      if (socket.readyState === WebSocket.OPEN) {
        socket.send(JSON.stringify(joinMessage));
      }
    }
  });
});
