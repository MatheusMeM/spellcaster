# Rodada 4 · o programa é o projetor

`projetor.html`: modelo 3D (three.js r128, PBR) de um laser RGB 10 W numa sala com névoa. Abrir com `#laser`, `#tras` ou `#dentro` pula o splash e escolhe a câmera.

- **Show**: a parede recebe o frame ILDA (mesmo parser e física da rodada 3, agora como textura); os feixes saem da abertura do modelo.
- **Trás = menu**: AC IN (liga/desliga), chave (arma), interlock (SCAN FAIL), DMX IN (endereço), ILDA IN (carrega `.ild`, aceita drop), ETHER (NDI · Spout · Art-Net · sACN), USB, VFD com quatro botões (MENU ▲ ▼ OK), ventoinha.
- **Dentro = preferências**: parafusos saem, tampa abre. Diodos 638/520/445 nm = limite e curva (γ) por cor; galvos = kpps; placa driver = buffer e velocidade; fonte e ventoinhas = temperatura; dicroicos = alinhamento.
- **Pino** (`../pino.js`): o mascote-menu. Publicar com `python design/build.py design/rodada4/projetor.html <saida>`, que inlina o Pino.
- **Voto** no fim da página, salvo em `localStorage` e em `moodboard/round4` no db do artifact.

Fotorrealismo sem asset: ambiente PMREM de estúdio procedural, ACES, sombras PCF, alumínio escovado com normal/roughness procedurais, chapas com chanfro, vidro físico nos dicroicos. Limite conhecido: sem SSAO nem DOF (não estão no build core).
