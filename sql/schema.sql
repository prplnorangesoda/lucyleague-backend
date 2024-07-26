DROP SCHEMA IF EXISTS testing CASCADE;
CREATE SCHEMA testing;

CREATE TABLE testing.users (
	id BIGSERIAL PRIMARY KEY,
	steamid BIGINT,
	username VARCHAR(50) UNIQUE NOT NULL,
	UNIQUE (username)
);

CREATE TABLE testing.teams (
	id BIGSERIAL PRIMARY KEY,
	team_name VARCHAR(200) NOT NULL
);

CREATE TABLE testing.userTeam (
	userid BIGSERIAL NOT NULL,
	teamid BIGSERIAL NOT NULL,
	CONSTRAINT FK_userTeam_user FOREIGN KEY (userid) references testing.users(id),
	CONSTRAINT FK_userTeam_team FOREIGN KEY (teamid) references testing.teams(id)
);
