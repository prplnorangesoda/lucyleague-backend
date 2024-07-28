CREATE TABLE IF NOT EXISTS users (
	id BIGSERIAL PRIMARY KEY,
	steamid VARCHAR(50) UNIQUE NOT NULL,
	username VARCHAR(50) UNIQUE NOT NULL,
	admin UNIQUE BOOLEAN NOT NULL,
	UNIQUE (username),
	UNIQUE (steamid)
);

CREATE TABLE IF NOT EXISTS teams (
	id BIGSERIAL PRIMARY KEY,
	team_name VARCHAR(200) NOT NULL
);

CREATE TABLE IF NOT EXISTS userTeam (
	userid BIGSERIAL NOT NULL,
	teamid BIGSERIAL NOT NULL,
	CONSTRAINT FK_userTeam_user FOREIGN KEY (userid) references users(id),
	CONSTRAINT FK_userTeam_team FOREIGN KEY (teamid) references teams(id)
);

CREATE TABLE IF NOT EXISTS authorizations (
	userid BIGSERIAL NOT NULL,
	token VARCHAR(50) NOT NULL,
	expires TIMESTAMPTZ NOT NULL,
	CONSTRAINT FK_authorization_user FOREIGN KEY (userid) references users(id)
);
