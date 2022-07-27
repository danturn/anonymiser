--
-- PostgreSQL database dump
--

-- Dumped from database version 14.4
-- Dumped by pg_dump version 14.4

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: orders; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.orders (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    product_id bigint NOT NULL
);


--
-- Name: orders_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

ALTER TABLE public.orders ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.orders_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- Name: products; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.products (
    id bigint NOT NULL,
    description text NOT NULL,
    price numeric(15,4) NOT NULL
);


--
-- Name: products_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

ALTER TABLE public.products ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.products_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- Name: users; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.users (
    id bigint NOT NULL,
    email character varying(255) NOT NULL,
    password character varying(255),
    last_login timestamp without time zone,
    inserted_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    first_name character varying(255),
    last_name character varying(255),
    deactivated boolean DEFAULT false NOT NULL,
    phone_number character varying(255),
    CONSTRAINT deactivated_users_cannot_have_real_passwords CHECK (((deactivated = false) OR ((deactivated = true) AND (password IS NOT NULL) AND ((password)::text = 'deleted'::text))))
);


--
-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

ALTER TABLE public.users ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.users_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- Data for Name: orders; Type: TABLE DATA; Schema: public; Owner: -
--

COPY public.orders (id, user_id, product_id) FROM stdin;
1	1	1
2	1	2
3	1	3
4	1	4
5	2	1
6	3	4
7	3	3
8	5	2
\.


--
-- Data for Name: products; Type: TABLE DATA; Schema: public; Owner: -
--

COPY public.products (id, description, price) FROM stdin;
1	c wschxirfe obod rs enaymlln	24.9900
2	r wcrpnyq znfd yb yxomvjim	34.9900
3	w dkvf ljadsx lzug	44.9900
4	xlntk	54.9900
\.


--
-- Data for Name: users; Type: TABLE DATA; Schema: public; Owner: -
--

COPY public.users (id, email, password, last_login, inserted_at, updated_at, first_name, last_name, deactivated, phone_number) FROM stdin;
1	17-durward_quo@gmail.com	not a password	2022-05-01 12:00:00	2020-01-06 11:22:00	2022-05-01 12:00:00	Maye	Kris	f	+447700921522
2	24-vida_nam@gmail.com	not a password	2022-05-02 12:00:00	2020-02-06 11:22:00	2022-05-02 12:00:00	Alberta	Hintz	f	+447700953977
3	29-harmon_voluptatem@gmail.com	not a password	2022-05-03 12:00:00	2020-03-06 11:22:00	2022-05-03 12:00:00	Creola	Schneider	f	+447700904699
4	37-seth_quis@hotmail.com	not a password	2022-05-04 12:00:00	2020-04-06 11:22:00	2022-05-04 12:00:00	Bertram	Brown	f	+447700916932
5	42-laurie_perferendis@gmail.com	not a password	2022-05-05 12:00:00	2020-05-06 11:22:00	2022-05-05 12:00:00	Eula	Shanahan	f	+447700932275
6	47-hanna_non@gmail.com	not a password	2022-05-06 12:00:00	2020-06-06 11:22:00	2022-05-06 12:00:00	Cordia	Walter	f	+447700944734
7	52-reba_ipsam@yahoo.com	not a password	2022-05-07 12:00:00	2020-07-06 11:22:00	2022-05-07 12:00:00	Claudia	Beatty	f	+447700936851
\.


--
-- Name: orders_id_seq; Type: SEQUENCE SET; Schema: public; Owner: -
--

SELECT pg_catalog.setval('public.orders_id_seq', 8, true);


--
-- Name: products_id_seq; Type: SEQUENCE SET; Schema: public; Owner: -
--

SELECT pg_catalog.setval('public.products_id_seq', 4, true);


--
-- Name: users_id_seq; Type: SEQUENCE SET; Schema: public; Owner: -
--

SELECT pg_catalog.setval('public.users_id_seq', 7, true);


--
-- Name: orders orders_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.orders
    ADD CONSTRAINT orders_pkey PRIMARY KEY (id);


--
-- Name: products products_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.products
    ADD CONSTRAINT products_pkey PRIMARY KEY (id);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: orders fk_product; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.orders
    ADD CONSTRAINT fk_product FOREIGN KEY (product_id) REFERENCES public.products(id);


--
-- Name: orders fk_user; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.orders
    ADD CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- PostgreSQL database dump complete
--

