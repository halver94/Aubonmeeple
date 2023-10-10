\c scraper;

update okkazeo_announce set oa_image = replace(oa_image, '.png', '.jpg') where oa_image like '%.png';