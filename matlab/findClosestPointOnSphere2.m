function [closestPoint,index] = findClosestPointOnSphere2(points, testPoint)
    % points: matrix with columns [azimuth, elevation, radius]
    % testPoint: vector with elements [azimuth, elevation, radius]

    % Convert azimuth and elevation to radians
    az_points = deg2rad(points(:, 1));
    el_points = deg2rad(points(:, 2));

    az_test = deg2rad(testPoint(1));
    el_test = deg2rad(testPoint(2));

    % Convert spherical coordinates to Cartesian coordinates
    [points_cart(:,1),points_cart(:,2),points_cart(:,3)] = sph2cart(points(:,1),points(:,2),points(:,3));
    [test_point_cart(:,1), test_point_cart(:,2), test_point_cart(:,3)] = sph2cart(testPoint(:,1),testPoint(:,2),testPoint(:,3));

    % Calculate distances in Cartesian coordinates
    distances = sqrt(sum((points_cart - test_point_cart).^2,2));
    % Find the index of the closest point
    [~, index] = min(distances);

    % Return the closest point
    closestPoint = points(index, :);
end