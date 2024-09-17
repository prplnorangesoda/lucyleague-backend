CREATE TABLE IF NOT EXISTS users (
	id BIGSERIAL PRIMARY KEY,
	steamid VARCHAR(50) UNIQUE NOT NULL,
	username VARCHAR(50) NOT NULL,
	avatarurl VARCHAR(200) NOT NULL,
	permissions BIGINT DEFAULT 0 NOT NULL,
	created_at TIMESTAMPTZ NOT NULL,
	UNIQUE (username),
	UNIQUE (steamid)
);

CREATE TABLE IF NOT EXISTS leagues (
  id BIGSERIAL PRIMARY KEY,
  name VARCHAR(50) NOT NULL,
	accepting_teams BOOLEAN DEFAULT FALSE NOT NULL,
	created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS teams (
	id BIGSERIAL PRIMARY KEY,
	leagueid BIGSERIAL NOT NULL,
	team_name VARCHAR(200) NOT NULL,
	created_at TIMESTAMPTZ NOT NULL,
	CONSTRAINT FK_teams_league FOREIGN KEY (leagueid) references leagues(id)
);

CREATE TABLE IF NOT EXISTS userTeam (
  leagueid BIGSERIAL NOT NULL,
	userid BIGSERIAL NOT NULL,
	teamid BIGSERIAL NOT NULL,
	created_at TIMESTAMPTZ NOT NULL,
  CONSTRAINT FK_userTeam_league FOREIGN KEY (leagueid) references leagues(id),
	CONSTRAINT FK_userTeam_user FOREIGN KEY (userid) references users(id),
	CONSTRAINT FK_userTeam_team FOREIGN KEY (teamid) references teams(id)
);

CREATE TABLE IF NOT EXISTS games (
	id BIGSERIAL NOT NULL,
	title VARCHAR(50),
	teamhomeid BIGSERIAL NOT NULL,
	teamawayid BIGSERIAL NOT NULL,
	created_at TIMESTAMPTZ NOT NULL,
	played_at TIMESTAMPTZ NOT NULL,
	CONSTRAINT FK_game_home FOREIGN KEY (teamhomeid) references teams(id),
	CONSTRAINT FK_game_away FOREIGN KEY (teamawayid) references teams(id)
);

CREATE TABLE IF NOT EXISTS authorizations (
	userid BIGSERIAL NOT NULL,
	token VARCHAR(50) NOT NULL,
	created_at TIMESTAMPTZ NOT NULL,
	expires TIMESTAMPTZ NOT NULL,
	CONSTRAINT FK_authorization_user FOREIGN KEY (userid) references users(id)
);
