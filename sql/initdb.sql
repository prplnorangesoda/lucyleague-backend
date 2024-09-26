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
	is_hidden BOOLEAN DEFAULT FALSE NOT NULL,
	created_at TIMESTAMPTZ NOT NULL
);
CREATE TABLE IF NOT EXISTS divisions (
  id BIGSERIAL PRIMARY KEY,
	leagueid BIGSERIAL,
  name VARCHAR(50) NOT NULL,
	created_at TIMESTAMPTZ NOT NULL,
	CONSTRAINT FK_divisions_league FOREIGN KEY (leagueid) references leagues(id)
);

CREATE TABLE IF NOT EXISTS division_admins (
	id BIGSERIAL PRIMARY KEY,
	divisionid BIGSERIAL NOT NULL,
	userid BIGSERIAL NOT NULL,
	relation VARCHAR(50) NOT NULL DEFAULT 'Admin',
	CONSTRAINT FK_division_admins_user FOREIGN KEY (userid) references users(id),
	CONSTRAINT FK_division_admins_division FOREIGN KEY (divisionid) references divisions(id)
);

CREATE TABLE IF NOT EXISTS teams (
	id BIGSERIAL PRIMARY KEY,
	leagueid BIGSERIAL NOT NULL,
	divisionid BIGSERIAL NOT NULL,
	team_tag VARCHAR(6) NOT NULL DEFAULT 'Tag',
	team_name VARCHAR(200) NOT NULL DEFAULT 'Unnamed',
	created_at TIMESTAMPTZ NOT NULL,
	CONSTRAINT FK_teams_league FOREIGN KEY (leagueid) references leagues(id),
	CONSTRAINT FK_teams_division FOREIGN KEY (divisionid) references divisions(id)
);

CREATE TABLE IF NOT EXISTS userTeam (
  leagueid BIGSERIAL NOT NULL,
	divisionid BIGSERIAL,
	userid BIGSERIAL NOT NULL,
	teamid BIGSERIAL NOT NULL,
	created_at TIMESTAMPTZ NOT NULL,
	ended_at TIMESTAMPTZ NOT NULL,
	is_leader BOOLEAN NOT NULL,
  CONSTRAINT FK_userTeam_league FOREIGN KEY (leagueid) references leagues(id),
	CONSTRAINT FK_userTeam_user FOREIGN KEY (userid) references users(id),
	CONSTRAINT FK_userTeam_team FOREIGN KEY (teamid) references teams(id)
);

CREATE TABLE IF NOT EXISTS games (
	id BIGSERIAL NOT NULL,
	title VARCHAR(50),
	leagueid BIGSERIAL NOT NULL,
	teamhomeid BIGSERIAL NOT NULL,
	teamawayid BIGSERIAL NOT NULL,
	created_at TIMESTAMPTZ NOT NULL,
	played_at TIMESTAMPTZ NOT NULL,
  CONSTRAINT FK_game_league FOREIGN KEY (leagueid) references leagues(id),
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

CREATE TABLE IF NOT EXISTS team_invites (
	leagueid BIGSERIAL NOT NULL,
	teamid BIGSERIAL NOT NULL,
	to_userid BIGSERIAL NOT NULL,
	from_userid BIGSERIAL NOT NULL,
	CONSTRAINT FK_team_invites_leagueid FOREIGN KEY (leagueid) references leagues(id),
	CONSTRAINT FK_team_invites_teamid FOREIGN KEY (teamid) references teams(id),
	CONSTRAINT FK_team_invites_to_userid FOREIGN KEY (to_userid) references users(id),
	CONSTRAINT FK_team_invites_from_userid FOREIGN KEY (from_userid) references users(id)
);


CREATE TABLE IF NOT EXISTS team_join_requests (
	leagueid BIGSERIAL NOT NULL,
	teamid BIGSERIAL NOT NULL,
	from_userid BIGSERIAL NOT NULL,
	CONSTRAINT team_join_requests_leagueid FOREIGN KEY (leagueid) references leagues(id),
	CONSTRAINT team_join_requests_teamid FOREIGN KEY (teamid) references teams(id),
	CONSTRAINT team_join_requests_from_userid FOREIGN KEY (from_userid) references users(id)
);
