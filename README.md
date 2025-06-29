# Como testar por agora:

## Requisitos
- Rust
- Cargo
- MySql

## Testar o programa
#### Criando a mysql database
Se for a primeira vez rodando, você deve primeiro criar o database do mysql (garanta que o serviço está ativo e executando).
Se desejar, você pode mudar o user e a password para acessar o database do mysql antes de criar efetivamente a database, basta alterar,
dentro da chamada da função "init_mysql_database" em /utils/src/main.rs, "nyoxon" pelo novo nome de usuário e "1234" pela nova senha. E também certifique-se de colocar a senha e usuário corretos
para o usuário com privilégios do seu sistema. Em /utils/src/main.rs basta mudar o primeiro "root" pelo nome do usuário com privilégios e o segundo
"root" pela senha desse usuário no mysql.

Agora, finalmente, dentro do diretório raiz faça:

```bash
cargo run -p utils
```

Outro dentro de utils:

```bash
cargo run
```

Se nada der errado, a database "auth_database" terá sido criada no mysql e um arquivo oculto ".env" no diretório raiz do projeto.

Se tudo der errado você pode ter que acabar criando o database na mão mesmo. Se esse for o caso, você pode ver como eu to fazendo para criar o database automaticamente na função "init_mysql_database" em /utils/src/lib.rs e/ou pedir ajuda pra alguma IA.

#### Compilar e executar o programa

Agora que a database foi criada e o arquivo .env existe dentro do diretorio raiz, basta fazer:

```bash
cargo run -p server
```

Estando no diretório raiz ou:

```bash
cargo run
```

Estando no diretório ./server, para criar o servidor web.

Agora em outro terminal/cmd, estando no diretório raiz, faça (se for fazer isso mesmo leia o comentário em ./client/src/main.rs):

```bash
cargo run -p client
```

Estando no diretório raiz ou:

```bash
cargo run
```

Estando no diretório ./client, para se conectar ao servidor antes criado.







![Contributors](https://img.shields.io/github/contributors/DevWorksAi/dw_web_server.svg)
